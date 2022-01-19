import React from 'react'
import type from '@/bindings/bindings'
import styles from './lobbies.module.scss'
import Lobby from './lobby'
import NotFound from './not_found'

interface LobbiesProps {
    lobbies : type.ExposedLobbyInfo[]
}

const Lobbies : React.FC<LobbiesProps> = ({lobbies}) => {
    return (
        <ol className={styles.lobbies}>
            {lobbies.length == 0 ? <NotFound/> : lobbies.map(lobby => <Lobby {...lobby}/>)}
        </ol>
    )
}

export default Lobbies