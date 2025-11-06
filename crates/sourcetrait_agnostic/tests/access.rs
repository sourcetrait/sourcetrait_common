#[cfg(test)]
mod tests {
    use std::{collections::HashSet, env};

    use sourcetrait_agnostic::{self as agnostic, prelude::*};
    use sourcetrait_testing::prelude::*;

    static TESTING: testing::Module = testing::module!(Integration);
    
    #[test]
    fn test_as_aid_display() {
        fn format_aid<AID: AsAID>(aid: AID) -> String {
            format!("{}", aid.as_aid())
        }
        
        let user = agnostic::PLATFORM.access().current_user().unwrap();
        assert!(!format_aid(&user).is_empty());
        assert!(!format_aid(user.to_id()).is_empty());
        assert!(!format_aid(user.to_id().as_aid()).is_empty());
        assert!(!format_aid(user).is_empty());
    }

    #[tested]
    fn test_current_user() {
        let _test = testing::test!();
        let user = agnostic::PLATFORM.access().current_user().unwrap();
        assert_eq!(user.username(), env!("USER"))
    }

    #[ignore]
    #[cfg(target_family = "unix")]
    #[tested]
    fn test_current_user_primary_group() {
        let _test = testing::test!();
        let user = agnostic::PLATFORM.access().current_user().unwrap();
        let group = agnostic::PLATFORM.access().user_primary_group(&user).unwrap().expect("capable");

        match agnostic::PLATFORM.os() {
            agnostic::Os::Linux => assert_eq!(group.groupname(), env!("USER")),
            agnostic::Os::MacOS => assert_eq!(group.groupname(), "staff"),
            _ => panic!("unsupported os"),
        }
    }
    
    #[tested]
    fn test_current_user_groups() {
        let test = testing::test!();
        let user = agnostic::PLATFORM.access().current_user().unwrap();
        let user_groups = agnostic::PLATFORM.access().user_groups(&user).unwrap();
        
        let users = user_groups.iter()
            .flat_map(|group| agnostic::PLATFORM.access().group_users(group).unwrap())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        
        let groups = users.iter()
            .flat_map(|user| agnostic::PLATFORM.access().user_groups(user).unwrap())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        
        assert_eq!(false, user_groups.is_empty());
        assert_eq!(false, groups.is_empty());
        assert_eq!(false, users.is_empty());
        
        let expected = if users.len() > 1 { &users[1] } else { &users[0] };
        let actual = agnostic::PLATFORM.access().user(&expected.to_id_key()).unwrap().unwrap();
        assert_eq!(expected, &actual);
        
        if test.is_env_debugging() {
            println!("[DEBUG] test_current_user_groups:");
            dbg!(user);
            dbg!(user_groups);
            dbg!(groups);
            dbg!(users);
        }
    }
}
