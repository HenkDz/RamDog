//! Persistência em %APPDATA%\RamDog\config.json

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::categories::Category;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Nomes de executáveis (minúsculo, com .exe) protegidos contra encerramento.
    pub locked: BTreeSet<String>,
    /// Override manual de categoria por nome de executável (minúsculo, com .exe).
    pub overrides: BTreeMap<String, Category>,
    pub refresh_ms: u64,
    pub confirm_kill: bool,
    /// Ocultar processos com menos que X MB (na métrica escolhida em `mem_metric`).
    pub min_mb: u32,
    pub view: ViewMode,
    pub show_system: bool,
    /// Qual número a coluna RAM mostra. Ver `MemMetric`.
    pub mem_metric: MemMetric,
    /// Mostrar as linhas sintéticas de kernel/compartilhado no topo da lista.
    pub show_kernel_rows: bool,
    /// Modo mini: HUD compacto, sem decoração de janela, só os medidores do topo.
    /// Persistido para o app reabrir no modo em que foi fechado.
    pub mini: bool,
    /// No modo mini, manter a janela por cima das outras.
    pub mini_on_top: bool,
}

/// Qual das três medidas de memória a coluna RAM exibe.
///
/// O padrão histórico do RamDog (e do Gerenciador de Tarefas) era `Private`, que exclui
/// tudo que é compartilhado entre processos — DLLs, seções compartilhadas, arquivos
/// mapeados. Numa máquina de 61 GB medida em 2026-08-21 isso somava 9,7 GB contra
/// 21,0 GB de working set: mais da metade da RAM dos processos ficava invisível.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum MemMetric {
    /// Working set: RAM física que o processo ocupa agora, incluindo páginas compartilhadas.
    /// Responde "quem devo matar". Superconta o compartilhado — a soma estoura o total.
    WorkingSet,
    /// Working set privado: só o que é exclusivo do processo. Soma abaixo do real, mas é a
    /// única base que fecha a conta contra o "em uso" sem dupla contagem.
    Private,
    /// Commit (pagefile usage): o que o processo reservou, esteja na RAM ou no disco.
    Commit,
}

impl MemMetric {
    pub const ALL: [MemMetric; 3] = [MemMetric::WorkingSet, MemMetric::Private, MemMetric::Commit];

    pub fn label(self) -> &'static str {
        match self {
            MemMetric::WorkingSet => "Working set",
            MemMetric::Private => "Private",
            MemMetric::Commit => "Commit",
        }
    }

    /// Rótulo curto para o cabeçalho da coluna.
    pub fn short(self) -> &'static str {
        match self {
            MemMetric::WorkingSet => "RAM",
            MemMetric::Private => "Private RAM",
            MemMetric::Commit => "Commit",
        }
    }

    pub fn tip(self) -> &'static str {
        match self {
            MemMetric::WorkingSet => concat!(
                "Physical RAM currently occupied, including shared pages (DLLs, shared memory, ",
                "mapped files). This is the right number for deciding what to terminate.\n\n",
                "A 50 MB DLL mapped into 30 processes counts in all 30, so the column total can ",
                "exceed total usage — the footer uses private memory for reconciliation."
            ),
            MemMetric::Private => concat!(
                "Only memory exclusive to the process — this is Task Manager's column.\n\n",
                "It excludes DLLs and shared memory, so it greatly understates processes such as ",
                "Chrome/Electron. In return, it is the only basis that sums without duplication."
            ),
            MemMetric::Commit => concat!(
                "Committed memory: what the process reserved, whether in RAM or the paging ",
                "file.\n\nIt anticipates memory pressure but does not say what is in RAM now."
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ViewMode {
    List,
    Tree,
    Category,
    Boot,
    Drains,
    Thermal,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            locked: BTreeSet::new(),
            overrides: BTreeMap::new(),
            refresh_ms: 1000,
            confirm_kill: true,
            min_mb: 0,
            view: ViewMode::List,
            show_system: true,
            mem_metric: MemMetric::WorkingSet,
            show_kernel_rows: true,
            mini: false,
            mini_on_top: true,
        }
    }
}

pub fn config_path() -> PathBuf {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library/Application Support"))
        .unwrap_or_else(|| PathBuf::from("."));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("RamDog").join("config.json")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let s = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, s).map_err(|e| e.to_string())
    }
}
