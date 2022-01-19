import React from "react"
import type from '@/bindings/bindings'
import styles from './lobbies.module.scss'

const Lobby : React.FC<type.ExposedLobbyInfo> = ({id, name, players, owner_id}) => {
    return (
        <li className={styles.lobby}>
            <p>#{id}</p>
            <p>{name}</p>
            <div className="spacer"></div>
            <p>{players.length} players</p>
        </li>
    )
}

export default Lobby