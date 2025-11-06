#[cfg(test)]
mod tests {
    use sourcetrait_agnostic::{self as agnostic, prelude::*};
    
    #[test]
    #[cfg_attr(not(feature = "test_with_gui"), ignore)]
    fn test_has_gui() {
        assert_eq!(true, agnostic::PLATFORM.ui().has_graphical().unwrap())
    }
    
    #[test]
    #[cfg_attr(not(feature = "test_with_terminal"), ignore)]
    fn test_has_terminal() {
        assert_eq!(true, agnostic::PLATFORM.ui().has_command_line().unwrap())
    }
    
    #[test]
    #[cfg_attr(not(all(feature = "test_with_terminal", feature = "test_with_gui")), ignore)]
    fn test_preference_has_both() {
        assert_eq!(agnostic::UserInterface::CLI, agnostic::PLATFORM.ui().preference().unwrap());
    }
    
    #[test]
    fn test_preference_set() {
        let original = agnostic::PLATFORM.ui().preference().unwrap();
        let other = match original {
            agnostic::UserInterface::CLI => agnostic::UserInterface::GUI,
            agnostic::UserInterface::GUI => agnostic::UserInterface::CLI,
            agnostic::UserInterface::None => agnostic::CrossError::err_unsupported().unwrap(),
        };
        
        if other == agnostic::UserInterface::GUI && !agnostic::PLATFORM.ui().has_graphical().unwrap() {
            // test that this should fail (gui is unavailable)
            assert!(matches!(agnostic::PLATFORM.ui().prefer(other), Err(agnostic::CrossError::Unavailable)));
        } else {
            agnostic::PLATFORM.ui().prefer(other).unwrap();
            assert_eq!(other, agnostic::PLATFORM.ui().preference().unwrap());
            
            agnostic::PLATFORM.ui().prefer(original).unwrap();
            assert_eq!(original, agnostic::PLATFORM.ui().preference().unwrap());
        }
    }
}
