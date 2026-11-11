#define MyAppName "Throne Rubezh"
#define MyAppExe "throne-rubezh.exe"

[Setup]
AppId={{B3F6A0C2-7E41-4D55-9C1A-5A2F1E7D9A10}
AppName={#MyAppName}
AppVersion=0.1.0
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=installer
OutputBaseFilename=Throne-Rubezh-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
DisableProgramGroupPage=yes

[Files]
Source: "dist\{#MyAppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; WorkingDir: "{app}"

[Run]
Filename: "{app}\{#MyAppExe}"; Description: "Launch the game"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent
