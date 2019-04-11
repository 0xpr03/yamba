import * as React from 'react';
import './App.css';

import logo from './logo.svg';

interface IState {
    results: string[]
}

interface Props {

}

class App extends React.Component<Props, IState> {
    constructor(props: any) {
        super(props);
        this.state = {
            results: []
        }
    }

    public componentDidMount(): void {
        console.log('');
        const request = new Request('http://localhost:8080/login', {
            body: 'username=root&password=TtmYpOJnBVqvEOntinDD',
            credentials: "include",
            headers: {
                'Content-type': 'application/x-www-form-urlencoded; charset=UTF-8'
            },
            method: 'POST'
        });
        fetch(request)
            .then(response => response.text())
            .then(text => this.setState({results: [text]}))
            .catch(error => error.toString());
    }

    public render() {
        return (
            <div className="App">
                <header className="App-header">
                    <img src={logo} className="App-logo" alt="logo"/>
                    <h1 className="App-title">Welcome to React</h1>
                </header>
                <p className="App-intro">
                    Server states: {this.state.results}
                </p>
            </div>
        );
    }
}

export default App;
