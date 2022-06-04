import type types from '@/bindings/bindings'

export type ServerCallback = (socket:Socket) => void
export type OnMessageCallback = (message:types.ServerMessage, socket:Socket) => void

export default class Socket {
    connection : WebSocket;
    onOpen? : ServerCallback
    reconnectionTimeout = 5000
    messageHandlers = new Map<string, OnMessageCallback>()

    // has to be run only in useEffect because we need the window to exist
    // to create a websocket and run updates
    constructor(serverUrl : string, onOpen : ServerCallback) {
        this.connection = new WebSocket(serverUrl);
        this.onOpen = onOpen
        this.setSocketHandlers()

        this.connection.onmessage = (event : MessageEvent) => {
            // bad implementation by ts, will read that: 
            // https://dev.to/codeprototype/safely-parsing-json-to-a-typescript-interface-3lkj 
            // some day
            let message = JSON.parse(event.data) as types.ServerMessage
            for (let [_, handler] of this.messageHandlers) {
                handler(message, this)
            }
        }
    }

    setSocketHandlers() {
        this.connection.onopen = () => {
            this.onOpen?.(this)
            console.info("Successfuly connected.")
        }
        this.connection.onclose = () => this.reconnect()
    }

    send(message : types.ClientMessage) {
        // might accidentally send a message while connecting
        if (this.connection.readyState == WebSocket.OPEN) {
            this.connection.send(JSON.stringify(message))
        }
    }

    reconnect() {
        console.info('Attempt to reconnect has begun.')
        // wait a lil to not kill cpu
        setTimeout(() => {
            this.connection = new WebSocket(this.connection.url)
            this.setSocketHandlers()
        }, this.reconnectionTimeout)
    }
}