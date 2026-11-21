use lotusair_bluetooth::BluetoothManager;
use lotusair_core::models::{NoiseControlMode, SpatialAudioMode};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let (manager, _) = BluetoothManager::new();
    manager.start().await;
    sleep(Duration::from_secs(3)).await;

    let state = manager.get_current_state().await;
    println!("Name: '{}', MAC: {}, Model: {:?}, AAP connected: {}", state.name, state.address, state.model, state.connected);
    println!("Battery: {:?}", state.battery);

    println!("ANC: {:?}", manager.set_noise_mode(NoiseControlMode::NoiseCancellation).await);
    println!("Transparency: {:?}", manager.set_noise_mode(NoiseControlMode::Transparency).await);
    println!("Adaptive: {:?}", manager.set_noise_mode(NoiseControlMode::Adaptive).await);
    println!("Off: {:?}", manager.set_noise_mode(NoiseControlMode::Off).await);
    println!("Restore ANC: {:?}", manager.set_noise_mode(NoiseControlMode::NoiseCancellation).await);
    println!("Spatial Fixed: {:?}", manager.set_spatial_audio_mode(SpatialAudioMode::Fixed).await);
    println!("Conversation: {:?}", manager.toggle_conversational_awareness(false).await);
    println!("Personalized volume: {:?}", manager.set_personalized_volume(true).await);
    println!("Loud noise reduction: {:?}", manager.set_loud_noise_reduction(true).await);
    println!("One-bud ANC: {:?}", manager.set_one_bud_anc(true).await);
    println!("Head gestures: {:?}", manager.set_head_gestures(true).await);
    println!("Auto ear detection: {:?}", manager.set_auto_ear_detection(true).await);
    println!("Auto takeover: {:?}", manager.set_auto_takeover(true).await);
    println!("Stem press left: {:?}", manager.set_stem_press_action("left".into(), "cyclenoisemodes".into()).await);
    println!("Stem press right: {:?}", manager.set_stem_press_action("right".into(), "cyclenoisemodes".into()).await);
    println!("Cycle modes: {:?}", manager.set_cycle_modes(true, true, true, false).await);
    println!("Case sounds: {:?}", manager.set_case_charging_sounds(true).await);
    println!("Case chime volume: {:?}", manager.set_case_chime_volume(80).await);
    println!("Acoustic fit test: {:?}", manager.run_acoustic_fit_test().await);
    println!("Adaptive blend: {:?}", manager.set_adaptive_blend(50).await);
    println!("Takeover: {:?}", manager.force_takeover_audio().await);
    println!("Keep alive: {:?}", manager.set_keep_alive(true).await);
}
