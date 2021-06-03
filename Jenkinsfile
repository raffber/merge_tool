pipeline {
    agent {
        dockerfile {
            dir 'jenkins'
        }
    }
    stages {
        stage('build-cli') {
            steps {
                sh 'cd cli && cargo build --release'
                archiveArtifacts(artifacts: 'cli/target/release/merge_tool_cli')
            }
        }
        stage('test-cli') {
            steps {
                sh 'cd cli && cargo test'
            }
        }
    }
}
