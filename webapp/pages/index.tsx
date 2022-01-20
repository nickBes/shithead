import React, { ChangeEvent, useEffect, useState, useRef, MouseEventHandler, Component } from 'react'
import FuzzySearch from 'fuzzy-search'
import {match, __} from 'ts-pattern'
import Lobbies from '@/components/lobby_list/lobbies'

// custom stuff
import styles from '@/styles/index.module.scss'
import { types } from '@/bindings/bindings'
import Creator from '@/components/LobbyCreator/creator'
import Socket, { OnMessageCallback, UpdateObject } from '@/client/socket'

// probably should save that in a .env file
// for both of the servers. though, this will do
// the work for now
const serverUrl = 'ws://localhost:7522'
const lobbySearchKeys = ['name', 'id']
let socket : Socket

const Index : React.FC = () => {
    // we want to re-render when lobbies change
    const [lobbies, setLobbies] = useState<types.ExposedLobbyInfo[]>([])
    const [clickedCreate, setClickedCreate] = useState(false)
    const [inputValue, setInputValue] = useState<string>()
    const [createdLobbyID, setCreatedLobbyID] = useState(-1)

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
    // configuration for constant server updates
    // about the lobby
    const updateObject : UpdateObject = {
        callback: (sk) => {
            sk.send("getLobbies")
        },
        timeInterval: 1000
    }
    // parse server messegaes and apply relevant methods
    const onMessage : OnMessageCallback = (message) => {
        match(message)
            .with({lobbies: __}, (msg) => {
                // update the search haystack and the lobby list
                setLobbies(msg.lobbies)
            })
            .with({joinLobby: __}, (msg) => {
                setCreatedLobbyID(msg.joinLobby)
            })
            .otherwise(msg => console.warn(`Don't have a matching pattern for message: ${JSON.stringify(msg)}`))
    }
    useEffect(() => {
        // open a socket when index loaded
        socket = new Socket(serverUrl, updateObject, onMessage)
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