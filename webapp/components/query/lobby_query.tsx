import React, { useState } from 'react'
import { types } from '@/bindings/bindings'
import styles from './query.module.scss'
import FuzzySearch from 'fuzzy-search'
import LobbyList from './lobby_list'

interface LobbyQueryProps {
    lobbies : types.ExposedLobbyInfo[]
}
const lobbySearchKeys = ['name', 'id']

const LobbyQuery : React.FC<LobbyQueryProps> = ({lobbies}) => {
    // we want to re-render when the user's input changes
    const [inputValue, setInputValue] = useState<string>()

    const filterLobbies = () => {
        if (inputValue === undefined) {
            lobbies
        }
        // creating a fuzzy search object to sort the lobbies
        // by relevancy to the user's input
        const fuzzy = new FuzzySearch(lobbies, lobbySearchKeys, {sort: true})
        return fuzzy.search(inputValue)
    }
    return (
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
                    <LobbyList lobbies={filterLobbies()}></LobbyList>
                </div>
            </form>
        </div>
    )
}

export default LobbyQuery