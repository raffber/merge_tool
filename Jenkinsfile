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
                sh 'cd cli/target/release && strip merge_tool_cli'
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
