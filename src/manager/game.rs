use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub fn is_game_running() -> bool {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    system.processes().values().any(|p| {
        p.name()
            .to_string_lossy()
            .eq_ignore_ascii_case("wotblitz.exe")
    })
}
