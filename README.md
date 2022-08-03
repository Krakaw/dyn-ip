# Dyn-ip

A dynamic IP service that uses Route53 as the provider.

### Usage

    # Configure your AWS credentials to allow modification of a Route53 domain only.
    cp .env.sample .env
    cargo run
    
    # Fetch your current domains
    curl localhost:8080/api/domains
    
    # Add a new subdomain using ip from client
    curl localhost:8080/api/domains?domain=subdomain -X POST
    # Add a new subdomain specifying an ip
    curl localhost:8080/api/domains?domain=subdomain&ip=127.0.0.1 -X POST

    # Update a subdomain with current client ip
    curl localhost:8080/api/domains/{domain_id_hash} -X PATCH

    # Update a subdomain with a specified ip
    curl localhost:8080/api/domains/{domain_id_hash}/{ip} -X PATCH

    # Test it
    dig subdomain.example.com
