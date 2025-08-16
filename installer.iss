[Setup]
OutputDir=target\inno
AppName=Reemap
AppId=Reemap
AppVersion=1.0.0
AppVerName=Reemap 1.0.0
WizardStyle=modern
OutputBaseFilename=Reemap 1.0.0 Setup
DefaultDirName={autopf}\Reemap
DefaultGroupName=Reemap
UninstallDisplayIcon={app}\reemap.exe
PrivilegesRequiredOverridesAllowed=dialog
PrivilegesRequired=admin
AppMutex=ReemapUniqueGuardMutexName
SetupMutex=ReemapInstallerUniqueGuardMutexName
SetupIconFile=resource\lurk.ico
LicenseFile=LICENSE-APACHE
WizardImageFile=resource\installer-banner.bmp
WizardSmallImageFile=resource\installer-icon.bmp
DisableWelcomePage=true

[Files]
Source: "target\release\reemap.exe"; DestDir: "{app}"

; [Tasks]
; Name: startup; Description: "Automatically start on login"; GroupDescription: "{cm:AdditionalIcons}"

[Icons]
Name: "{group}\Reemap"; Filename: "{app}\reemap.exe"
; Name: "{autostartup}\Reemap"; Filename: "{app}\reemap.exe"; Tasks: startup
