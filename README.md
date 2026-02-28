# simai-player-rs (Proto 2)

A rhythm game prototype built in Rust using the [Bevy](https://bevyengine.org/) engine. It parses `simai` chart formats (`maidata.txt`) and renders a circular gameplay field, heavily inspired by [maiノーツ](https://mai-notes.com/).

> **Note:** This project is currently WIP.

## Roadmap

**Gameplay Mechanics**
- [ ] **Slide Notes**: Implement complex pathing and delayed movement for star notes along the screen.
- [ ] **Touch Hold Notes**: Add logic for Touch notes that require the player to hold the center area after the triangles meet.

**Audio & Visuals**
- [ ] **Hit Effects & Particles**: Spawn visual feedback (e.g., explosions, rings) when a note is successfully hit.
- [ ] **Audio Synchronization**: Fine-tune audio playback to perfectly match the Bevy `Time` resource and handle offset calibration.

**Core & Systems**
- [ ] **Parser Improvements**: Enhance the parser to accurately handle a wider range of standard `simai` notation and edge cases. 
- [ ] **CLI Configuration**: Add a command-line interface for adjusting game options and startup parameters (e.g., passing the chart path directly).
