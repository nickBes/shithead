import Socket from "@/client/socket"
import React, { FormEvent } from "react"
import styles from './creator.module.scss'

interface LobbyCreatorProps {
    socket:Socket
}

const LobbyCreator : React.FC<LobbyCreatorProps> = ({socket}) => {
    const createLobby = (event : FormEvent<HTMLFormElement>) => {
        event?.preventDefault()
        let formData = new FormData(event.currentTarget)
        if (formData.has('lobbyName')) {
            let lobbyName = formData.get('lobbyName')
            if (typeof lobbyName == "string") {
                socket.send({
                    createLobby: {
                        lobby_name: lobbyName
                    }
                })
            }
        }
    }

    return (
        <div className={styles.creator}>
            <form className={styles.creatorForm} onSubmit={(event) => createLobby(event)}>
                <input autoComplete="off" name='lobbyName' placeholder='Lobby Name' type="text"/>
                <button type="submit">Create New Lobby</button>
            </form>
        </div>
    )
}

export default LobbyCreator