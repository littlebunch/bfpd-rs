table! {
    derivations (id) {
        id -> Integer,
        code -> Varchar,
        description -> Mediumtext,
    }
}

table! {
    foods (id) {
        id -> Integer,
        publication_date -> Datetime,
        modified_date -> Datetime,
        available_date -> Datetime,
        upc -> Varchar,
        fdc_id -> Varchar,
        description -> Varchar,
        food_group_id -> Integer,
        brand_id -> Integer,
        datasource -> Varchar,
        serving_size -> Nullable<Double>,
        serving_unit -> Nullable<Varchar>,
        serving_description -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        ingredients -> Nullable<Mediumtext>,
    }
}

table! {
    food_groups (id) {
        id -> Integer,
        description -> Varchar,
    }
}

table! {
    brands (id) {
        id -> Integer,
        owner -> Varchar,
        brand -> Nullable<Varchar>,
        subbrand -> Nullable<Varchar>,
    }
}

table! {
    nutrients (id) {
        id -> Integer,
        nutrientno -> Varchar,
        description -> Varchar,
        unit -> Varchar,
    }
}

table! {
    nutrient_data (id) {
        id -> Integer,
        value -> Double,
        portion_value -> Double,
        standard_error -> Nullable<Double>,
        minimum -> Nullable<Double>,
        maximum -> Nullable<Double>,
        median -> Nullable<Double>,
        derivation_id -> Integer,
        nutrient_id -> Integer,
        food_id -> Integer,
    }
}

joinable!(foods -> food_groups (food_group_id));
joinable!(foods -> brands (brand_id));
joinable!(nutrient_data -> derivations (derivation_id));
joinable!(nutrient_data -> foods (food_id));
joinable!(nutrient_data -> nutrients (nutrient_id));

allow_tables_to_appear_in_same_query!(
    derivations,
    foods,
    food_groups,
    brands,
    nutrients,
    nutrient_data,
);
