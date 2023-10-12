//! Pre-build entities that can be used in the public API

// As public API objects, these entities would probably
// only represent blueprints for creating actual scene entities.

// They would do nothing, just hold some data.

// They would probably need to implement a trait that the scene will use to
// add them to the renderer state machine.

trait EntityBlueprint {
    fn name(&self) -> &str;
    fn id(&self) -> &str;
    fn description(&self) -> &str;
}

// gaze.moveToBack();
// gaze.moveToFront();
// gaze.moveForward();
// gaze.moveBackward();
// gaze.hide();
// gaze.show();

