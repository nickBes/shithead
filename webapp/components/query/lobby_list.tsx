import React from 'react'
import type from '@/bindings/bindings'
import styles from './query.module.scss'
import LobbyListItem from './lobby_list_item'
import NotFound from './not_found'

interface LobbyListProps {
    lobbies : type.ExposedLobbyInfo[]
}

const LobbyList : React.FC<LobbyListProps> = ({lobbies}) => {
    return (
        <ol className={styles.lobbies}>
            {lobbies.length == 0 ? <NotFound/> : lobbies.map(lobby => <LobbyListItem key={lobby.id} {...lobby}/>)}
        </ol>
    )
}

export default LobbyList