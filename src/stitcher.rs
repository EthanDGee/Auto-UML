use crate::diagram::Diagram;

struct File {
    name: String,
    diagram: Diagram,
}

struct Directory {
    name: String,
    sub_directory: Vec<Directory>,
    files: Vec<File>,
    diagram: Diagram,
}

fn build(path: String) -> Directory {
    // walk through the files for every file build directory and every sub directory recurse and
    // add them to the directorys
    //
    // before merging all sub direcotrys classes/functions should have the directory appended to
    // the front of their names. We need to handle imports
    //
    //when striching it ais also important to focus on the actual type this is stored in the .lang
    //(LangCOnfig) assigned to each diagram
    //
    // finally link all the parts keeping mind their paths so that identically named calsses that
    // belong to seperate files/directorys don't get mixed.
    todo!()
}
