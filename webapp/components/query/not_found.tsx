import React from "react"
import styles from './query.module.scss'

const NotFound : React.FC = () => {
    return (
        <h1 className={styles.notFound}>There Are No Lobbies</h1>
    )
}

export default NotFound