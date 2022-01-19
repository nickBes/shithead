import React, { ChangeEvent, useEffect, useState, useRef } from 'react'
import FuzzySearch from 'fuzzy-search'
import {match, __} from 'ts-pattern'
import Lobbies from '@/components/Lobbies/lobbies'

// custom stuff
import styles from '@/styles/index.module.scss'
import { clientMessageToJSON, parseMessageEvent } from '@/client/messages'
import { types } from '@/bindings/bindings'

// probably should save that in a .env file
// for both of the servers. though, this will do
// the work for now
const serverUrl = 'ws://localhost:7522'
const globalUpdatesTime = 5000
const lobbySearchKeys = ['name', 'id']

const Index : React.FC = () => {
    // we want to re-render when lobbies change
    const [lobbies, setLobbies] = useState<types.ExposedLobbyInfo[]>([])
    const [inputValue, setInputValue] = useState<string>()

    let socket : WebSocket
    let globalUpdatesInterval : NodeJS.Timer

    const filterLobbies = () => {
        if (inputValue === undefined) {
            lobbies
        }
        // creating a fuzzy search object to sort the lobbies
        // by relevancy to the user's input
        const fuzzy = new FuzzySearch(lobbies, lobbySearchKeys, {sort: true})
        return fuzzy.search(inputValue)
    }

    useEffect(() => {
        // open a socket when index loaded
        socket = new WebSocket(serverUrl)

        // here we open an interval to get updates about
        // the lobby
        socket.onopen = (event) => {
            globalUpdatesInterval = setInterval(() => {
                socket.send(clientMessageToJSON("getLobbies"))
            }, globalUpdatesTime)
        }
        // parse server messegaes and apply relevant methods
        socket.onmessage = (event) => {
            const message = parseMessageEvent(event)
            match(message)
                .with({lobbies: __}, (msg) => {
                    // update the search haystack and the lobby list
                    setLobbies(msg.lobbies)
                })
                .otherwise(msg => console.warn(`No matching pattern for message: ${JSON.stringify(msg)}}`))
        }
        socket.onclose = (event) => {
            clearInterval(globalUpdatesInterval)
        }
    }, [])
    return (
        <>
            <main className={styles.main}>
                <nav className={styles.nav}>ShitHead</nav>
                <div className={styles.formWrap}>
                    <form className={styles.form}>
                        <label className={styles.inputLabel} htmlFor='search'>Find a Lobby</label>
                        <input autoComplete='off' 
                                className={styles.search} 
                                id="search" type='text' 
                                placeholder='Lobby Name' 
                                // force to re-render the lobbie list
                                onChange={(event) => setInputValue(event.target.value)}/>
                        <Lobbies lobbies={filterLobbies()}></Lobbies>
                    </form>
                </div>
            </main>
        </>
    )
}

export default Index