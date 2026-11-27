//! Dessin des disques hors du fil de l'interface.
//! Un seul disque à la fois : un pas plus récent chasse ceux qui attendent.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use image::RgbaImage;

use crate::bodies::{HeightMap, RingZone};
use crate::disk::{self, DiskInput};
use crate::ephemeris::Orientation;

pub struct DiskJob {
    pub token: u64,
    pub id: u64,
    pub orientation: Orientation,
    pub flattening: f64,
    pub albedo: [u8; 3],
    pub texture: Option<Arc<RgbaImage>>,
    /// Chargée sur le fil de rendu si `texture` est vide.
    pub texture_path: Option<PathBuf>,
    pub lon_offset_turns: f64,
    pub bump: Option<Arc<HeightMap>>,
    /// Chargée sur le fil de rendu si `bump` est vide.
    pub bump_path: Option<PathBuf>,
    pub rings: &'static [RingZone],
    pub draw_axis: bool,
    pub radius_px: f64,
    pub max_side: i32,
}

pub struct DiskDone {
    pub token: u64,
    pub id: u64,
    pub image: RgbaImage,
    pub loaded_texture: Option<Arc<RgbaImage>>,
    pub loaded_bump: Option<Arc<HeightMap>>,
}

struct Queue {
    jobs: VecDeque<DiskJob>,
    closed: bool,
}

pub struct RasterPump {
    queue: Arc<Mutex<Queue>>,
    wake: Arc<Condvar>,
    done_rx: std::sync::mpsc::Receiver<DiskDone>,
    current_token: u64,
    /// Compteur lu seulement par les tests, pour attendre la fin d'un dessin.
    #[cfg(test)]
    completed: Arc<AtomicU64>,
    join: Option<JoinHandle<()>>,
}

impl RasterPump {
    pub fn new() -> Self {
        let queue = Arc::new(Mutex::new(Queue {
            jobs: VecDeque::new(),
            closed: false,
        }));
        let wake = Arc::new(Condvar::new());
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        #[cfg(test)]
        let completed = Arc::new(AtomicU64::new(0));
        #[cfg(test)]
        let completed_thread = Some(Arc::clone(&completed));
        #[cfg(not(test))]
        let completed_thread = None;
        let queue_thread = Arc::clone(&queue);
        let wake_thread = Arc::clone(&wake);
        let join = std::thread::Builder::new()
            .name("raster".to_owned())
            .spawn(move || worker(queue_thread, wake_thread, done_tx, completed_thread))
            .expect("le fil de rendu démarre");
        Self {
            queue,
            wake,
            done_rx,
            current_token: 0,
            #[cfg(test)]
            completed,
            join: Some(join),
        }
    }

    pub fn bump_token(&mut self) -> u64 {
        self.current_token = self.current_token.wrapping_add(1);
        self.current_token
    }

    pub fn submit(&self, job: DiskJob) {
        {
            let mut queue = self.queue.lock().expect("file de rendu");
            let token = job.token;
            queue.jobs.retain(|queued| queued.token == token);
            queue.jobs.push_back(job);
        }
        self.wake.notify_one();
    }

    /// Images du jeton courant. Les jetons périmés sont écartés.
    pub fn poll(&self) -> Vec<DiskDone> {
        let mut ready = Vec::new();
        while let Ok(done) = self.done_rx.try_recv() {
            if done.token == self.current_token {
                ready.push(done);
            }
        }
        ready
    }
}

impl Drop for RasterPump {
    fn drop(&mut self) {
        {
            let mut queue = self.queue.lock().expect("file de rendu");
            queue.closed = true;
            queue.jobs.clear();
        }
        self.wake.notify_one();
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

fn worker(
    queue: Arc<Mutex<Queue>>,
    wake: Arc<Condvar>,
    done_tx: std::sync::mpsc::Sender<DiskDone>,
    completed: Option<Arc<AtomicU64>>,
) {
    loop {
        let job = {
            let mut queue = queue.lock().expect("file de rendu");
            loop {
                if let Some(job) = next_job(&mut queue.jobs) {
                    break job;
                }
                if queue.closed {
                    return;
                }
                queue = wake.wait(queue).expect("file de rendu");
            }
        };
        let token = job.token;
        let id = job.id;
        let mut job = job;
        resolve_maps(&mut job);
        let loaded_texture = job.texture.clone();
        let loaded_bump = job.bump.clone();
        let image = render(&job);
        if done_tx
            .send(DiskDone {
                token,
                id,
                image,
                loaded_texture,
                loaded_bump,
            })
            .is_err()
        {
            return;
        }
        if let Some(completed) = &completed {
            completed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Ne garde que le jeton le plus récent, puis sort le premier de ceux-là.
fn next_job(jobs: &mut VecDeque<DiskJob>) -> Option<DiskJob> {
    let latest = jobs.iter().map(|job| job.token).max()?;
    jobs.retain(|job| job.token == latest);
    jobs.pop_front()
}

fn resolve_maps(job: &mut DiskJob) {
    if job.texture.is_none() {
        if let Some(path) = job.texture_path.take() {
            match crate::bodies::load_path(&path) {
                Some(image) => job.texture = Some(Arc::new(image)),
                None => eprintln!("Photo illisible : {}", path.display()),
            }
        }
    }
    if job.bump.is_none() {
        if let Some(path) = job.bump_path.take() {
            match crate::bodies::load_bump_file(&path) {
                Some(map) => job.bump = Some(Arc::new(map)),
                None => eprintln!("Relief illisible : {}", path.display()),
            }
        }
    }
}

fn render(job: &DiskJob) -> RgbaImage {
    disk::render_disk(
        &DiskInput {
            orientation: &job.orientation,
            flattening: job.flattening,
            albedo: job.albedo,
            texture: job.texture.as_deref(),
            lon_offset_turns: job.lon_offset_turns,
            bump: job.bump.as_deref(),
            rings: job.rings,
            draw_axis: job.draw_axis,
        },
        job.radius_px,
        job.max_side,
    )
}

/// Pastille au centre du ciel. Aucun clic n'est pris.
pub fn paint_busy(painter: &egui::Painter, area: egui::Rect) {
    let galley = painter.layout(
        "Génération en cours...".to_owned(),
        egui::FontId::proportional(20.0),
        egui::Color32::from_rgb(235, 240, 245),
        800.0,
    );
    let pad = egui::vec2(18.0, 10.0);
    let rect = egui::Rect::from_center_size(area.center(), galley.size() + pad * 2.0);
    painter.rect(
        rect,
        8.0,
        egui::Color32::from_rgba_unmultiplied(4, 10, 28, 230),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 150, 60)),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        egui::pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        ),
        galley,
        egui::Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ephemeris::{Orientation, Vec3};
    use std::time::{Duration, Instant};

    fn plain_job(token: u64, id: u64) -> DiskJob {
        DiskJob {
            token,
            id,
            orientation: Orientation {
                x_axis: Vec3::new(1.0, 0.0, 0.0),
                y_axis: Vec3::new(0.0, 1.0, 0.0),
                z_axis: Vec3::new(0.0, 0.0, 1.0),
                to_observer: Vec3::new(1.0, 0.0, 0.0),
                to_sun: Vec3::new(1.0, 0.0, 0.0),
            },
            flattening: 0.0,
            albedo: [180, 40, 40],
            texture: None,
            texture_path: None,
            lon_offset_turns: 0.0,
            bump: None,
            bump_path: None,
            rings: &[],
            draw_axis: false,
            radius_px: 8.0,
            max_side: 64,
        }
    }

    #[test]
    fn next_job_drops_an_older_token() {
        let mut jobs = VecDeque::from([
            plain_job(1, 1),
            plain_job(1, 2),
            plain_job(2, 3),
            plain_job(2, 4),
        ]);
        let first = next_job(&mut jobs).unwrap();
        assert_eq!((first.token, first.id), (2, 3));
        let second = next_job(&mut jobs).unwrap();
        assert_eq!((second.token, second.id), (2, 4));
        assert!(next_job(&mut jobs).is_none());
    }

    #[test]
    fn a_small_disk_comes_back() {
        let mut pump = RasterPump::new();
        let token = pump.bump_token();
        pump.submit(plain_job(token, 7));
        let done = wait_ready(&pump, 1);
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].token, token);
        assert_eq!(done[0].id, 7);
        assert!(done[0].image.width() >= 2);
        assert!(done[0].image.height() >= 2);
    }

    #[test]
    fn a_stale_token_is_discarded() {
        let mut pump = RasterPump::new();
        let old = pump.bump_token();
        pump.submit(plain_job(old, 1));
        let _current = pump.bump_token();
        let _ = wait_completed(&pump.completed, 1);
        assert!(pump.poll().is_empty());
    }

    fn wait_ready(pump: &RasterPump, n: u64) -> Vec<DiskDone> {
        let _ = wait_completed(&pump.completed, n);
        let done = pump.poll();
        assert_eq!(done.len(), n as usize, "image attendue");
        done
    }

    fn wait_completed(completed: &AtomicU64, n: u64) -> u64 {
        let start = Instant::now();
        while completed.load(Ordering::Relaxed) < n {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "le fil de rendu n'a pas fini"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        completed.load(Ordering::Relaxed)
    }
}
