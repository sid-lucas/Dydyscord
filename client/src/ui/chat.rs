use openmls::group::GroupId;

struct Group {
    id: GroupId,
    name: String,
}

pub fn browse_groups(groups: Vec<(GroupId, String)>) {
    let groups: Vec<Group> = groups
        .into_iter()
        .map(|(id, name)| Group { id, name })
        .collect();

    let options: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();

    let selection = inquire::Select::new("Select a group:", options)
        .prompt()
        .expect("An error occurred");

    match groups.into_iter().find(|g| g.name == selection) {
        Some(group) => show_chat(group),
        None => println!("Group not found"),
    }
}

fn show_chat(group: Group) {
    println!("Showing chat for group ID: {:?}", group.id);
}
