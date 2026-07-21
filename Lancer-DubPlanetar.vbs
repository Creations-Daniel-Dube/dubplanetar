' Lance DubPlanetar sans fenêtre console (double-clic).
' Utilise pythonw du système si disponible, sinon python / py.
Option Explicit

Dim fso, sh, root, cmd, rc
Set fso = CreateObject("Scripting.FileSystemObject")
Set sh = CreateObject("WScript.Shell")

root = fso.GetParentFolderName(WScript.ScriptFullName)
sh.CurrentDirectory = root

If fso.FileExists(root & "\.venv\Scripts\pythonw.exe") Then
  cmd = """" & root & "\.venv\Scripts\pythonw.exe"" """ & root & "\launch_dubplanetar.py"""
ElseIf fso.FileExists(root & "\.venv\Scripts\python.exe") Then
  cmd = """" & root & "\.venv\Scripts\python.exe"" """ & root & "\launch_dubplanetar.py"""
ElseIf HaveCmd("pythonw") Then
  cmd = "pythonw """ & root & "\launch_dubplanetar.py"""
ElseIf HaveCmd("python") Then
  cmd = "python """ & root & "\launch_dubplanetar.py"""
ElseIf HaveCmd("py") Then
  cmd = "py -3 """ & root & "\launch_dubplanetar.py"""
Else
  MsgBox "Python introuvable." & vbCrLf & vbCrLf & _
         "Double-cliquez d'abord sur Installer-DubPlanetar.bat" & vbCrLf & _
         "apres avoir installe Python 3.11+ depuis python.org" & vbCrLf & _
         "(cochez Add python.exe to PATH).", _
         vbExclamation, "DubPlanetar"
  WScript.Quit 1
End If

' 0 = fenêtre cachée ; False = ne pas attendre la fin (UI indépendante)
rc = sh.Run(cmd, 0, False)
WScript.Quit 0

Function HaveCmd(name)
  Dim e
  On Error Resume Next
  e = sh.Run("cmd /c where " & name & " >nul 2>&1", 0, True)
  On Error GoTo 0
  HaveCmd = (e = 0)
End Function
