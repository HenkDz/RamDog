//! Catálogo do que cada processo do Windows faz, por que está aberto e se dá para matar.
//!
//! Existe porque a pergunta que o Gerenciador de Tarefas nunca responde é a única que
//! importa: "não sei o que diabos isso faz, posso fechar?". Sem essa resposta o usuário
//! ou mata algo essencial ou deixa lixo rodando por medo.
//!
//! O texto é curto de propósito — cabe numa linha do painel de detalhes. Nada aqui é
//! consultado por amostra; é uma tabela estática, custo zero em tempo de execução.

/// O que acontece se o processo for encerrado.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Risk {
    /// Encerrar é seguro: no máximo você perde o que estava fazendo nele.
    Safe,
    /// O Windows reabre sozinho em segundos — matar quase nunca resolve nada.
    Respawns,
    /// Derruba a sessão ou o sistema inteiro (tela azul / logoff forçado).
    Fatal,
}

impl Risk {
    pub fn label(self) -> &'static str {
        match self {
            Risk::Safe => "safe to terminate",
            Risk::Respawns => "respawns automatically",
            Risk::Fatal => "DO NOT terminate",
        }
    }

    pub fn dot(self) -> &'static str {
        match self {
            Risk::Safe => "🟢",
            Risk::Respawns => "🟡",
            Risk::Fatal => "🔴",
        }
    }

    pub fn color(self) -> egui::Color32 {
        match self {
            Risk::Safe => egui::Color32::from_rgb(90, 220, 130),
            Risk::Respawns => egui::Color32::from_rgb(230, 190, 80),
            Risk::Fatal => egui::Color32::from_rgb(235, 90, 90),
        }
    }

    pub fn tip(self) -> &'static str {
        match self {
            Risk::Safe => "Terminating it will not break Windows. You only lose unsaved work in this program.",
            Risk::Respawns => "Windows restarts this process automatically. Terminating it frees RAM for a few seconds, then it returns.",
            Risk::Fatal => "Critical process: terminating it causes a blue screen (CRITICAL_PROCESS_DIED) or immediately ends your session.",
        }
    }
}

pub struct Known {
    /// O que o processo faz, em uma frase.
    pub what: &'static str,
    /// Por que ele está aberto agora — a pergunta que ninguém responde.
    pub why: &'static str,
    pub risk: Risk,
}

const fn k(what: &'static str, why: &'static str, risk: Risk) -> Known {
    Known { what, why, risk }
}

/// Ficha do processo pelo nome do executável (minúsculo, com ou sem `.exe`).
pub fn lookup(name_lower: &str) -> Option<Known> {
    let b = name_lower.strip_suffix(".exe").unwrap_or(name_lower);
    Some(match b {
        // ── Núcleo da sessão: matar qualquer um destes derruba o Windows ──────────────
        "system" => k(
            "The Windows kernel and drivers, grouped into a fictional process.",
            "It exists since boot. It is not a program: it is Windows.",
            Risk::Fatal,
        ),
        "registry" => k(
            "Stores the Windows Registry loaded in memory.",
            "Always open — the entire system reads configuration from it.",
            Risk::Fatal,
        ),
        "memory compression" => k(
            "Compresses rarely used RAM pages instead of sending them to disk.",
            "Grows when RAM is tight. It saves memory; it is not waste.",
            Risk::Fatal,
        ),
        "secure system" | "lsaiso" => k(
            "Virtualization-based isolation (VBS/Credential Guard) — protects credentials from the rest of the system.",
            "Enabled because Virtualization-based Security is active on this machine.",
            Risk::Fatal,
        ),
        "smss" => k(
            "Session Manager: the first user-mode process during boot.",
            "Creates each session and then exits — which is why it often appears as an already-exited parent.",
            Risk::Fatal,
        ),
        "csrss" => k(
            "Win32 runtime subsystem: consoles, process creation, and process termination.",
            "One per session, since login. There will always be at least two.",
            Risk::Fatal,
        ),
        "wininit" => k(
            "Session 0 initialization: starts services.exe, lsass.exe, and the session manager.",
            "It is the grandparent of every Windows service. It uses little memory itself — the large tree value is the sum of its children.",
            Risk::Fatal,
        ),
        "winlogon" => k(
            "Handles login, screen locking, and Ctrl+Alt+Del.",
            "One per interactive session, since you powered on the PC.",
            Risk::Fatal,
        ),
        "services" => k(
            "Service Control Manager: starts, stops, and supervises every Windows service.",
            "Parent of almost every svchost.exe — hence the large RAM value in the tree view.",
            Risk::Fatal,
        ),
        "lsass" => k(
            "Local Security Authority: validates passwords, tokens, and security policies.",
            "Always open. It is also a favorite target for credential theft.",
            Risk::Fatal,
        ),
        "fontdrvhost" => k(
            "Hosts the font driver outside the kernel, isolated for security.",
            "Starts with the graphical session.",
            Risk::Fatal,
        ),
        "dwm" => k(
            "Desktop Window Manager: composites everything you see, including transparency and shadows.",
            "Without it there is no desktop. Its RAM is mostly video buffers.",
            Risk::Fatal,
        ),
        "logonui" => k(
            "Draws the login and lock screens.",
            "Appears when the session is locked.",
            Risk::Fatal,
        ),

        // ── Reabrem sozinhos ─────────────────────────────────────────────────────────
        "explorer" => k(
            "The desktop, taskbar, and folder windows.",
            "It also hosts tray icons and shell extensions from other programs — so it grows over time.",
            Risk::Respawns,
        ),
        "svchost" => k(
            "A generic shell that hosts Windows services: the name alone says nothing.",
            "What matters is which service is inside — RamDog shows that in the details.",
            Risk::Respawns,
        ),
        "sihost" => k(
            "Shell infrastructure: context menus, notifications, and taskbar actions.",
            "One per user session.",
            Risk::Respawns,
        ),
        "ctfmon" => k(
            "Text input: virtual keyboard, languages, and handwriting recognition.",
            "Starts as soon as any text field exists.",
            Risk::Respawns,
        ),
        "runtimebroker" => k(
            "Monitors Store app permissions (camera, microphone, files).",
            "One per modern app that is open. Several at once is normal.",
            Risk::Respawns,
        ),
        "wmiprvse" => k(
            "WMI provider: answers inventory and monitoring queries about the machine.",
            "Starts on demand and exits after a few idle minutes. Antivirus software and RamDog make these queries.",
            Risk::Respawns,
        ),
        "searchhost" | "searchapp" => k(
            "Start menu search and the taskbar search box.",
            "Preloaded so it opens instantly when you press the Windows key.",
            Risk::Respawns,
        ),
        "searchindexer" => k(
            "Indexes files and email so search responds quickly.",
            "Works in bursts after many files change.",
            Risk::Respawns,
        ),
        "startmenuexperiencehost" => k(
            "Draws the Start menu.",
            "Preloaded for speed, even when the menu is closed.",
            Risk::Respawns,
        ),
        "shellexperiencehost" => k(
            "Notification center, clock, and visual parts of the taskbar.",
            "Always present in the graphical session.",
            Risk::Respawns,
        ),
        "textinputhost" => k(
            "Virtual keyboard, emoji panel, and text suggestions.",
            "Preloaded so the emoji panel (Win+.) opens without delay.",
            Risk::Respawns,
        ),
        "applicationframehost" => k(
            "Provides the window frame for Store apps.",
            "One per modern app that is open.",
            Risk::Respawns,
        ),
        "dllhost" => k(
            "Hosts COM components without their own process, such as file thumbnails.",
            "Appears and exits as Explorer needs to generate previews.",
            Risk::Respawns,
        ),
        "taskhostw" => k(
            "Runs scheduled tasks that are libraries rather than programs.",
            "Starts when Task Scheduler triggers something.",
            Risk::Respawns,
        ),
        "spoolsv" => k(
            "Print spooler.",
            "Always open, even without a printer installed.",
            Risk::Respawns,
        ),
        "audiodg" => k(
            "Isolates driver audio effects from the sound service.",
            "Starts when something plays audio.",
            Risk::Respawns,
        ),
        "conhost" | "openconsole" => k(
            "Classic console window for command-line programs.",
            "One per legacy terminal program that is running.",
            Risk::Respawns,
        ),
        "systemsettings" => k(
            "The Windows Settings app.",
            "Remains suspended in the background after it is closed.",
            Risk::Respawns,
        ),
        "appactions" => k(
            "App actions suggested by Windows (share, open with).",
            "Shell component, starts on demand.",
            Risk::Respawns,
        ),
        "widgets" | "widgetservice" => k(
            "Taskbar widgets panel (weather, news).",
            "Can be disabled in taskbar settings.",
            Risk::Respawns,
        ),
        "phoneexperiencehost" => k(
            "Phone Link app.",
            "Starts automatically if you have linked a phone.",
            Risk::Respawns,
        ),
        "wudfhost" => k(
            "Hosts user-mode drivers (printers, biometrics, USB peripherals).",
            "One per connected device class.",
            Risk::Respawns,
        ),
        // ── Defender e segurança ─────────────────────────────────────────────────────
        "msmpeng" => k(
            "Microsoft Defender engine: scans files in real time.",
            "Always open. RAM rises during builds and large downloads — excluding project folders helps a lot.",
            Risk::Respawns,
        ),
        "nissrv" => k(
            "Defender network inspection.",
            "Companion to MsMpEng.",
            Risk::Respawns,
        ),
        "securityhealthservice" | "securityhealthsystray" => k(
            "Windows Security Center: the shield icon and status panel.",
            "Permanent system service.",
            Risk::Respawns,
        ),
        "mpdefendercoreservice" => k(
            "Defender core service, separate from the scanning engine.",
            "Part of real-time protection.",
            Risk::Respawns,
        ),

        // ── Atualização e nuvem ──────────────────────────────────────────────────────
        "onedrive" => k(
            "Syncs folders with OneDrive.",
            "Starts with Windows. Can be disabled if you do not use Microsoft's cloud.",
            Risk::Safe,
        ),
        "usocoreworker" | "mousocoreworker" => k(
            "Windows Update orchestrator.",
            "Starts to check, download, or prepare updates. Exits afterward.",
            Risk::Respawns,
        ),
        "tiworker" | "trustedinstaller" => k(
            "Windows Modules Installer: applies updates and components.",
            "Uses CPU in bursts after an update. It is temporary.",
            Risk::Respawns,
        ),
        "compattelrunner" => k(
            "Application compatibility telemetry.",
            "Runs as a scheduled task in the background.",
            Risk::Safe,
        ),

        // ── Programas comuns ─────────────────────────────────────────────────────────
        "chrome" | "msedge" | "firefox" | "brave" | "opera" | "vivaldi" => k(
            "Browser. Each tab, extension, and site has its own isolated process.",
            "The sum of the children is the real usage — one process alone says little.",
            Risk::Safe,
        ),
        "code" | "cursor" | "devenv" | "rider64" | "idea64" | "pycharm64" => k(
            "Code editor. Language servers and extensions run in separate processes.",
            "The children usually use more resources than the window itself.",
            Risk::Safe,
        ),
        "node" => k(
            "JavaScript runtime: development server, build tool, or agent.",
            "Check the command line in the details to see which project started it.",
            Risk::Safe,
        ),
        "python" | "python3" | "pythonw" => k(
            "Python interpreter: script, server, or tool.",
            "The command line in the details says which script is running.",
            Risk::Safe,
        ),
        "rustc" | "cargo" | "rust-analyzer" => k(
            "Rust toolchain tool: compilation or code analysis in the editor.",
            "Appears during a build or with a Rust project open in the editor.",
            Risk::Safe,
        ),
        "steam" | "steamwebhelper" => k(
            "Steam client. steamwebhelper draws the interface as an embedded browser.",
            "Starts with Windows by default; it can be disabled in Steam settings.",
            Risk::Safe,
        ),
        "discord" | "spotify" | "slack" | "teams" | "ms-teams" | "whatsapp" | "telegram" => k(
            "Desktop app built on an embedded browser (Electron).",
            "Usually starts with Windows and stays in the system tray.",
            Risk::Safe,
        ),
        "nvcontainer" | "nvdisplay.container" => k(
            "NVIDIA service container: telemetry, overlay, and driver control.",
            "Installed with the graphics driver.",
            Risk::Respawns,
        ),
        _ => return None,
    })
}
