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
            }
        }
        stage('test-cli') {
            steps {
                sh 'cd cli && cargo test'
            }
        }
    }
}
