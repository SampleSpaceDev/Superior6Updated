-- Initial database schema for Superior 6

-- Users table
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       name VARCHAR(255) NOT NULL,
                       display_name VARCHAR(100) NOT NULL,
                       email VARCHAR(255) NOT NULL UNIQUE,
                       password_hash VARCHAR(255) NOT NULL,
                       is_admin BOOLEAN DEFAULT FALSE,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                       updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Gameweeks table
CREATE TABLE gameweeks (
                           id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                           week_number INTEGER NOT NULL,
                           season VARCHAR(20) NOT NULL,
                           deadline TIMESTAMP WITH TIME ZONE NOT NULL,
                           is_active BOOLEAN DEFAULT FALSE,
                           is_completed BOOLEAN DEFAULT FALSE,
                           created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                           updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                           UNIQUE(week_number, season)
);

-- Fixtures table
CREATE TABLE fixtures (
                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          gameweek_id UUID NOT NULL REFERENCES gameweeks(id) ON DELETE CASCADE,
                          home_team VARCHAR(255) NOT NULL,
                          away_team VARCHAR(255) NOT NULL,
                          kickoff_time TIMESTAMP WITH TIME ZONE NOT NULL,
                          home_score INTEGER,
                          away_score INTEGER,
                          fixture_order INTEGER NOT NULL,
                          created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                          updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                          CONSTRAINT valid_fixture_order CHECK (fixture_order >= 1 AND fixture_order <= 6)
);

-- Predictions table
CREATE TABLE predictions (
                             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                             user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                             fixture_id UUID NOT NULL REFERENCES fixtures(id) ON DELETE CASCADE,
                             home_score_prediction INTEGER NOT NULL,
                             away_score_prediction INTEGER NOT NULL,
                             points_awarded INTEGER DEFAULT 0,
                             created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                             updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                             UNIQUE(user_id, fixture_id)
);

-- Gameweek scores table
CREATE TABLE gameweek_scores (
                                 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                 user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                 gameweek_id UUID NOT NULL REFERENCES gameweeks(id) ON DELETE CASCADE,
                                 total_points INTEGER DEFAULT 0,
                                 exact_scores INTEGER DEFAULT 0,
                                 correct_results INTEGER DEFAULT 0,
                                 created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                                 updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                                 UNIQUE(user_id, gameweek_id)
);

-- Season scores table
CREATE TABLE season_scores (
                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               season VARCHAR(20) NOT NULL,
                               total_points INTEGER DEFAULT 0,
                               total_exact_scores INTEGER DEFAULT 0,
                               total_correct_results INTEGER DEFAULT 0,
                               gameweeks_played INTEGER DEFAULT 0,
                               created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                               updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                               UNIQUE(user_id, season)
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_gameweeks_season ON gameweeks(season, week_number);
CREATE INDEX idx_gameweeks_active ON gameweeks(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_fixtures_gameweek ON fixtures(gameweek_id);
CREATE INDEX idx_predictions_user ON predictions(user_id);
CREATE INDEX idx_predictions_fixture ON predictions(fixture_id);
CREATE INDEX idx_gameweek_scores_user ON gameweek_scores(user_id);
CREATE INDEX idx_gameweek_scores_gameweek ON gameweek_scores(gameweek_id);
CREATE INDEX idx_season_scores_user_season ON season_scores(user_id, season);

-- Trigger function for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply triggers
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_gameweeks_updated_at BEFORE UPDATE ON gameweeks FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_fixtures_updated_at BEFORE UPDATE ON fixtures FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_predictions_updated_at BEFORE UPDATE ON predictions FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_gameweek_scores_updated_at BEFORE UPDATE ON gameweek_scores FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_season_scores_updated_at BEFORE UPDATE ON season_scores FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();