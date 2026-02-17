use openmls::group::GroupId;

pub fn show_groups(groups: Vec<(GroupId, String)>) {
    struct Group {
        id: GroupId,
        name: String,
    }

    let groups: Vec<Group> = groups
        .into_iter()
        .map(|(id, name)| Group { id, name })
        .collect();

    let options: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();

    let selection = inquire::Select::new("Select a group:", options)
        .prompt()
        .expect("An error occurred");

    match groups.into_iter().find(|g| g.name == selection) {
        Some(group) => println!("You selected group with ID: {:?}", group.id),
        None => println!("Group not found"),
    }
}
