use javipiv2::core::installation_state::InstallationState;

#[test]
fn los_estados_recibidos_son_distintos() {
    let estados = [
        InstallationState::Pending,
        InstallationState::Running,
        InstallationState::Completed,
        InstallationState::Failed,
        InstallationState::Skipped,
    ];
    for (indice, estado) in estados.iter().enumerate() {
        assert_eq!(*estado, estados[indice]);
        for otro in &estados[indice + 1..] {
            assert_ne!(estado, otro);
        }
    }
}
