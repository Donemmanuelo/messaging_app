let encoded = encoder.encode(&REGISTRY.gather(), &mut buffer).map_err(|e| format!("Encoding error: {}", e))?;
let result = String::from_utf8(buffer).map_err(|e| format!("UTF8 error: {}", e))?; 