use mik_sdk_macros::Type;

// Basic enum - should compile and generate FromJson, ToJson, OpenApiSchema
#[derive(Type)]
enum Status {
    Active,
    Inactive,
    Pending,
}

// Enum with multi-word variants (PascalCase -> snake_case)
#[derive(Type)]
enum UserRole {
    SuperAdmin,
    RegularUser,
    GuestUser,
}

// Enum with custom rename
#[derive(Type)]
enum PaymentStatus {
    #[field(rename = "PAID")]
    Paid,
    #[field(rename = "PENDING")]
    Pending,
    #[field(rename = "FAILED")]
    Failed,
}

fn main() {}
