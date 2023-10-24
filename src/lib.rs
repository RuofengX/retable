mod db;
mod two_key;
pub mod err;
pub mod atom;
pub use db::Database;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::db::{Database, Atom};
    use bson::{Bson, doc};

    #[test]
    fn test_database() {
        let mut database = Database::new();

        // 添加一些原子
        let mut atom1 = Atom::new(1, "prop1".to_string(), Bson::String(String::from("v1")));
        let atom2 = Atom::new(1, "prop2".to_string(), Bson::String(String::from("v2")));
        let atom3 = Atom::new(2, "prop1".to_string(), Bson::String(String::from("v3")));
        let atom4 = Atom::new(2, "prop2".to_string(), Bson::String(String::from("v4")));

        database.insert_atom(atom1.clone()).unwrap();
        database.insert_atom(atom2.clone()).unwrap();
        database.insert_atom(atom3.clone()).unwrap();
        database.insert_atom(atom4.clone()).unwrap();

        // 测试get_props方法
        let props = database.get_props(1);
        assert_eq!(props.len(), 2);
        assert!(props.contains(&&atom1));
        assert!(props.contains(&&atom2));

        // 测试get_id方法
        let id_props = database.get_entities("prop1".to_string());
        assert_eq!(id_props.len(), 2);
        assert!(id_props.contains(&&atom1));
        assert!(id_props.contains(&&atom3));

        // 测试get方法
        let ent = database.get(1, "prop1".to_string()).unwrap();
        assert_eq!(ent, &atom1);

        // 测试update方法
        let new_prop = Bson::Document(doc! { "key": "value" });
        atom1.value = new_prop.clone();
        let updated_atom = database.update(1, "prop1".to_string(), new_prop.clone());
        assert_eq!(updated_atom, Some(&atom1));
        assert_eq!(atom1.value, new_prop);

        // 测试save方法
        let path = Path::new("./save.ron");
        assert!(database.save(path).is_ok());

        // 测试load方法
        let database_ng = Database::load(path).unwrap();
        assert_eq!(database_ng.get(1, "prop1".to_string()).unwrap(), &atom1);

    }
}

