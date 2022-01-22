import types from '@/bindings/bindings'

export type ServerCallback = (socket:Socket) => void
export type OnMessageCallback = (message:types.ServerMessage, socket:Socket) => void

// an object for setting up timed communication
// with the server: for every timeInterval that is passed
// the callback method will be called
export interface UpdateObject {
    callback: ServerCallback | undefined
    timeInterval: number
}

export default class Socket {
    connection : WebSocket;
    onOpen : ServerCallback
    updateInterval : number
    reconnectionTimeout = 5000
    updateObject : UpdateObject
    onMessage : OnMessageCallback

    // has to be run only in useEffect because we need the window to exist
    // to create a websocket and run updates
    constructor(serverUrl : string, onOpen : ServerCallback, updateObject : UpdateObject, onMessage:OnMessageCallback) {
        this.connection = new WebSocket(serverUrl);
        this.onMessage = onMessage
        this.updateObject = updateObject
        this.onOpen = onOpen

        this.setSocketHandlers()
    }

    setSocketHandlers() {
        this.connection.onopen = () => {
            this.onOpen(this)
            console.info("Successfuly connected.")
            this.update(this.updateObject)
        }
        this.connection.onmessage = (event : MessageEvent) => {
            // bad implementation by ts, will read that: 
            // https://dev.to/codeprototype/safely-parsing-json-to-a-typescript-interface-3lkj 
            // some day
            this.onMessage(JSON.parse(event.data) as types.ServerMessage, this)
        }
        this.connection.onclose = () => this.reconnect()
    }

    send(message : types.ClientMessage) {
        // might accidentally send a message while connecting
        if (this.connection.readyState == WebSocket.OPEN) {
            this.connection.send(JSON.stringify(message))
        }
    }

    update(updateObject : UpdateObject) {
        this.updateInterval = window.setInterval(() => {
            updateObject.callback?.(this)
        }, 
        updateObject.timeInterval)
    }

    stopUpdating() {
        clearInterval(this.updateInterval)
    }

    reconnect() {
        console.info('Attempt to reconnect has begun.')
        // stop doing stuff on the socket
        this.stopUpdating()
        // wait a lil to not kill cpu
        setTimeout(() => {
            this.connection = new WebSocket(this.connection.url)
            this.setSocketHandlers()
        }, this.reconnectionTimeout)
    }
}