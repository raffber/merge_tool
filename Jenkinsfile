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
        stage('build-artifacts-windows') {
            steps {
                sh 'cd cli && cargo build --target x86_64-pc-windows-gnu --release'
                archiveArtifacts(artifacts: 'cli/target/x86_64-pc-windows-gnu/release/merge_tool_cli.exe')
            }
        }
    }
}
