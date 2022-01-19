import React, { ChangeEvent, useEffect, useState, useRef, MouseEventHandler, Component } from 'react'
import FuzzySearch from 'fuzzy-search'
import {match, __} from 'ts-pattern'
import Lobbies from '@/components/Lobbies/lobbies'

// custom stuff
import styles from '@/styles/index.module.scss'
import { clientMessageToJSON, parseMessageEvent } from '@/client/messages'
import { types } from '@/bindings/bindings'
import Creator from '@/components/LobbyCreator/creator'

// probably should save that in a .env file
// for both of the servers. though, this will do
// the work for now
const serverUrl = 'ws://localhost:7522'
const globalUpdatesTime = 5000
const lobbySearchKeys = ['name', 'id']
let socket : WebSocket

const Index : React.FC = () => {
    // we want to re-render when lobbies change
    const [lobbies, setLobbies] = useState<types.ExposedLobbyInfo[]>([])
    const [clickedCreate, setClickedCreate] = useState(false)
    const [inputValue, setInputValue] = useState<string>()
    const [createdLobbyID, setCreatedLobbyID] = useState(-1)

    let globalUpdatesInterval : NodeJS.Timer

    const toggleCreateLobby = () => setClickedCreate(prev => !prev)

    const filterLobbies = () => {
        if (inputValue === undefined) {
            lobbies
        }
        // creating a fuzzy search object to sort the lobbies
        // by relevancy to the user's input
        const fuzzy = new FuzzySearch(lobbies, lobbySearchKeys, {sort: true})
        return fuzzy.search(inputValue)
    }
    const startGlobalUpdate = () => setInterval(() => {
        socket.send(clientMessageToJSON("getLobbies"))
    }, globalUpdatesTime)
    useEffect(() => {
        // open a socket when index loaded
        socket = new WebSocket(serverUrl)
        // here we open an interval to get updates about
        // the lobby
        socket.onopen = (event) => {
            globalUpdatesInterval = startGlobalUpdate()
        }
        // parse server messegaes and apply relevant methods
        socket.onmessage = (event) => {
            const message = parseMessageEvent(event)
            match(message)
                .with({lobbies: __}, (msg) => {
                    // update the search haystack and the lobby list
                    setLobbies(msg.lobbies)
                })
                .with({'joinLobby': __}, (msg) => {
                    console.log(msg.joinLobby)
                    setCreatedLobbyID(msg.joinLobby)
                })
                .otherwise(msg => console.warn(`No matching pattern for message: ${JSON.stringify(msg)}}`))
        }
        socket.onclose = (event) => {
            clearInterval(globalUpdatesInterval)
            setTimeout(() => {
                socket = new WebSocket(serverUrl)
            }, globalUpdatesTime)
            globalUpdatesInterval = startGlobalUpdate()
            console.info('reopened a socket')
        }
    }, [])
    return (
        <>
            <main className={styles.main}>
                <nav className={styles.nav}>ShitHead</nav>
                <div className={styles.formWrap}>
                    <form className={styles.form}>
                        <div className={styles.lobbyWrap}>
                            <label className={styles.inputLabel} htmlFor='search'>Find a Lobby</label>
                            <input autoComplete='off' 
                                    className={styles.search} 
                                    id="search" type='text' 
                                    placeholder='Lobby Name' 
                                    // force to re-render the lobbie list
                                    onChange={(event) => setInputValue(event.target.value)}/>
                            <Lobbies lobbies={filterLobbies()}></Lobbies>
                        </div>
                        <button className={styles.createLobby} onClick={event => {
                            event.preventDefault()
                            toggleCreateLobby()
                        }}>{clickedCreate ? "Undo" : "Create Lobby"}</button>
                    </form>
                </div>
                {clickedCreate ? <Creator socket={socket}/> : ''}
            </main>
        </>
    )
}

export default Index