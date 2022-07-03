pipeline {
    agent {
        dockerfile {
            dir 'jenkins'
        }
    }
    stages {
        stage('build') {
            steps {
                sh 'cargo build --release'
                sh 'cd target/release && strip merge_tool'
                archiveArtifacts(artifacts: 'target/release/merge_tool')
            }
        }
        stage('test') {
            steps {
                sh 'cd cli && cargo test'
            }
        }
        stage('build-windows') {
            steps {
                sh 'cargo build --target x86_64-pc-windows-gnu --release'
                archiveArtifacts(artifacts: 'target/x86_64-pc-windows-gnu/release/merge_tool.exe')
            }
        }
    }
}
