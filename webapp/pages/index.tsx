import React, { useEffect, useState } from 'react'
import {match, __} from 'ts-pattern'
import LobbyQuery from '@/components/query/lobby_query'

// custom stuff
import styles from '@/styles/index.module.scss'
import { types } from '@/bindings/bindings'
import LobbyCreator from '@/components/lobby_creator/creator'
import Socket, { OnMessageCallback, ServerCallback, UpdateObject } from '@/client/socket'

// probably should save that in a .env file
// for both of the servers. though, this will do
// the work for now
const serverUrl = 'ws://localhost:7522'
let socket : Socket
// configuration for constant server updates
// about the lobby
const updateLobbyObject : UpdateObject = {
    callback: (sk) => {
        sk.send("getLobbies")
    },
    timeInterval: 1000
}
type GameStates = 'inLobby' | 'inGame' | 'createLobby' | 'inQuery'

const Index : React.FC = () => {
    // we want to re-render when lobbies/game-state change
    const [lobbies, setLobbies] = useState<types.ExposedLobbyInfo[]>([])
    const [gameState, setGameState] = useState<GameStates>('inQuery')

    // when socket opens set the state to inQuery
    const onOpen : ServerCallback = () => setGameState('inQuery')
    // parse server messegaes and apply relevant methods
    const onMessage : OnMessageCallback = (message, sk) => {
        match(message)
            .with({lobbies: __}, (msg) => {
                // update the search haystack and the lobby list
                setLobbies(msg.lobbies)
            })
            .with({joinLobby: __}, (msg) => {
                // stops getting new lobbt lists
                sk.updateObject.callback = undefined
                sk.stopUpdating()
                setGameState("inLobby")
            })
            .otherwise(msg => console.warn(`Don't have a matching pattern for message: ${JSON.stringify(msg)}`))
    }
    useEffect(() => {
        // open a socket when index loaded
        socket = new Socket(serverUrl, onOpen, updateLobbyObject, onMessage)
    }, [])
    return (
        <>
            <main className={styles.main}>
                <nav className={styles.nav}>ShitHead</nav>
                {
                    // render components per the game states
                    match(gameState)
                        .with("inQuery", () => {
                            return (
                                <>
                                    <LobbyQuery lobbies={lobbies}/>
                                    <button className={styles.createLobby} 
                                            onClick={event => {
                                                event.preventDefault()
                                                setGameState("createLobby")
                                            }}>Create Lobby</button>
                                </>
                            )
                        })
                        .with("createLobby", () => {
                            return (
                                <>
                                    <LobbyCreator socket={socket}/>
                                    <button className={styles.createLobby}
                                            onClick={event => {
                                                event.preventDefault()
                                                setGameState("inQuery")
                                            }}>Undo</button>
                                </>
                            )
                        })
                        // will be implemented later
                        .with("inLobby", () => console.log("LOBBY"))
                        .otherwise(() => console.log('Something went wrong.'))
                }
            </main>
        </>
    )
}

export default Index