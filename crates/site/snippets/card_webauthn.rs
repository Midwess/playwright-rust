let creds = context.credentials();
creds.install().await?;
// Register a passkey, then drive navigator.credentials in-page.
let passkey = creds.create("example.com", None).await?;
let registered = creds.get(None).await?;
creds.delete(&passkey.id).await?;
