    #!/bin/bash

    set -e  # Exit on error

    echo "=========================================="
    echo "Setting up movve monorepo..."
    echo "=========================================="
    echo ""

    # Copy environment file
    if [ ! -f .env ]; then
        cp .env.template .env
        echo "✅ Created .env file from template"
        echo ""
        echo "⚠️  IMPORTANT: Please update .env with your Supabase credentials:"
        echo "   1. Go to https://app.supabase.com/project/[YOUR-PROJECT]/settings/api"
        echo "   2. Copy your project URL and API keys"
        echo "   3. Go to https://app.supabase.com/project/[YOUR-PROJECT]/settings/database"
        echo "   4. Copy your database connection string"
        echo "   5. Update the .env file with these values"
        echo ""
        echo "Press Enter to continue after updating .env..."
        read -r
    else
        echo "✅ .env file already exists"
    fi

    # Load environment variables
    if [ -f .env ]; then
        export $(cat .env | grep -v '^#' | xargs)
    fi

    # Check if DATABASE_URL is set
    if [ -z "$DATABASE_URL" ]; then
        echo ""
        echo "❌ DATABASE_URL not found in .env file"
        echo ""
        echo "Please add your Supabase database connection string to .env:"
        echo "DATABASE_URL=postgresql://postgres:[YOUR-PASSWORD]@db.[YOUR-PROJECT-REF].supabase.co:5432/postgres"
        echo ""
        exit 1
    fi

    echo ""
    echo "=========================================="
    echo "Installing Rust Tools..."
    echo "=========================================="
    echo ""

    # Install cargo-make if not installed
    if ! command -v cargo-make &> /dev/null; then
        echo "Installing cargo-make..."
        cargo install cargo-make
        echo "✅ cargo-make installed"
    else
        echo "✅ cargo-make already installed"
    fi

    # Install cargo-watch for hot reloading
    if ! command -v cargo-watch &> /dev/null; then
        echo "Installing cargo-watch..."
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "Detected macOS, applying AppKit framework fix..."
            RUSTFLAGS="-C link-arg=-framework -C link-arg=AppKit" cargo install cargo-watch
        else
            cargo install cargo-watch
        fi
        echo "✅ cargo-watch installed"
    else
        echo "✅ cargo-watch already installed"
    fi

    # Install sqlx-cli
    if ! command -v sqlx &> /dev/null; then
        echo "Installing sqlx-cli..."
        cargo install sqlx-cli --no-default-features --features postgres
        echo "✅ sqlx-cli installed"
    else
        echo "✅ sqlx-cli already installed"
    fi

    # Install tmux if not available

    echo ""
    echo "=========================================="
    echo "Optional Tools"
    echo "=========================================="
    echo ""

    # Check for psql (optional but useful)
    if ! command -v psql &> /dev/null; then
        echo "⚠️  psql (PostgreSQL client) not found"
        echo "   This is optional but useful for database shell access"
        echo ""
        echo "   To install:"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "   brew install postgresql@18"
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            echo "   sudo apt-get install postgresql-client  # Ubuntu/Debian"
            echo "   sudo yum install postgresql             # RHEL/CentOS"
        fi
        echo ""
    else
        echo "✅ psql is installed"
    fi

    # Install Supabase CLI (optional)
    if ! command -v supabase &> /dev/null; then
        echo "⚠️  Supabase CLI not found (optional)"
        echo ""
        echo "Would you like to install Supabase CLI? (y/n)"
        echo "This enables extra features like schema diffing and type generation"
        read -r install_supabase
        if [ "$install_supabase" = "y" ]; then
            echo "Installing Supabase CLI..."
            if command -v brew &> /dev/null; then
                brew install supabase/tap/supabase
                echo "✅ Supabase CLI installed via Homebrew"
            elif command -v npm &> /dev/null; then
                echo "Installing via npm..."
                npm install -g supabase
                echo "✅ Supabase CLI installed via npm"
            else
                echo "❌ Neither Homebrew nor npm found. Please install manually:"
                echo "   Visit: https://supabase.com/docs/guides/cli"
            fi
        else
            echo "Skipping Supabase CLI installation"
        fi
    else
        echo "✅ Supabase CLI is installed"
    fi

    echo ""
    echo "=========================================="
    echo "Database Setup"
    echo "=========================================="
    echo ""

    # Test database connection
    echo "Testing database connection..."
    if sqlx database create 2>&1 | grep -q "already exists"; then
        echo "✅ Connected to Supabase database"
    elif sqlx database create &> /dev/null; then
        echo "✅ Database created successfully"
    else
        echo "❌ Failed to connect to database"
        echo ""
        echo "Please check:"
        echo "1. Your DATABASE_URL is correct in .env"
        echo "2. Your Supabase project is running"
        echo "3. Your IP is allowed in Supabase (if connection pooling is disabled)"
        echo ""
        exit 1
    fi

    # Run database migrations
    echo ""
    echo "Running database migrations..."
    if cargo make db-migrate; then
        echo "✅ All migrations completed successfully"
    else
        echo "❌ Migration failed"
        echo ""
        echo "Common issues:"
        echo "1. Check if migrations exist in apps/auth/migrations and apps/driver/migrations"
        echo "2. Verify DATABASE_URL in .env is correct"
        echo "3. Check Supabase project status at https://app.supabase.com"
        echo ""
        exit 1
    fi

    echo ""
    echo "=========================================="
    echo "Setup Complete! 🎉"
    echo "=========================================="
    echo ""
    echo "Next steps:"
    echo "  1. Run 'cargo make run-all' to start all services"
    echo "  2. Run 'cargo make db-help' to see all database commands"
    echo ""
    echo "Useful commands:"
    echo "  cargo make run-all          - Start all services"
    echo "  cargo make db-migrate       - Run migrations"
    echo "  cargo make db-migrate-info  - Check migration status"
    echo "  cargo make db-help          - Show all database commands"
    echo ""
    echo "Documentation:"
    echo "  Supabase Dashboard: https://app.supabase.com"
    echo "  Database Migrations: apps/auth/migrations, apps/driver/migrations"
    echo ""