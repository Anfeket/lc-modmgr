const { invoke } = window.__TAURI__.tauri
const { open } = window.__TAURI__.dialog

let mods = document.getElementById("modlist")

function maxId() {
	let setups = document.getElementById("setuplist")
	let maxId = -1
	for (let setup of setups.children) {
		let id = setup.id.split("setup")[1]
		maxId = Math.max(maxId, id)
	}
	return maxId + 1
}
function addSetup(setup) {
	let setups = document.getElementById("setuplist")
	let element = document.createElement('button')
	let id = maxId()
	element.classList.add('setupbtn')
	element.id = "setup" + id
	element.textContent = setup.name
	setups.append(element)
	globalsetups[id] = setup
	element.addEventListener("click", () => {
		selectSetup(id)
	})
}

function notification(text) {
	if (document.getElementById("notifdisable").checked == true) {
		return
	}
	let area = document.getElementById("notifications")
	let element = document.createElement('p')
	element.textContent = text
	area.append(element)
	element.classList.add("notifshow")
	function deleteNotif() {
		element.classList.remove("notifshow")
		element.classList.add("notifhide")
		setTimeout(() => { element.remove() }, 500)
	}
	element.addEventListener("click", deleteNotif)
	setTimeout(deleteNotif, 8000)
}

function showSettings() {
	document.getElementById("settingswindow").hidden = false
}
function hideSettings() {
	document.getElementById("settingswindow").hidden = true
}
document.getElementById("settings").addEventListener("click", showSettings)
document.getElementById("closesettings").addEventListener("click", hideSettings)

let selectedSetup = null
let globalsetups = {}
function selectSetup(setup) {
	if (selectedSetup != null) {
		document.getElementById("setup" + selectedSetup).classList.remove("selected")
	}
	document.getElementById("setuptitle").contentEditable = true
	selectedSetup = setup
	document.getElementById("setup" + setup).classList.add("selected")
	document.getElementById("setuptitle").textContent = globalsetups[setup].name
	let mods = ""
	for (let mod of globalsetups[setup].mods) {
		mods += mod.name + "\n"
	}
	document.getElementById("modlist").textContent = mods
}
function removeSetup(id) {
	if (selectedSetup == null) {
		notification("No Setup Selected!")
		return
	}
	selectedSetup = null
	document.getElementById("setuptitle").textContent = "No Setup Selected"
	document.getElementById("setuptitle").contentEditable = false
	document.getElementById("setup" + id).remove()
	let name = globalsetups[id].name
	invoke("remove_setup", { name: name })
	notification(globalsetups[id].name + " removed")
	delete globalsetups[id]
}
document.getElementById("removesetup").addEventListener("click", () => {
	removeSetup(selectedSetup)
})
function newSetup() {
	invoke("new_setup").then((data) => {
		notification("Created a new setup")
		addSetup(data)
	}).catch((e) => {
		console.log(e)
		notification(e)
	})
}
document.getElementById("newsetup").addEventListener("click", newSetup)

function loadSetup() {
	if (selectedSetup == null) {
		notification("No Setup Selected!")
		return
	}
	let id = selectedSetup
	let name = globalsetups[id].name
	invoke("load_setup", { name }).then(() => {
		notification("Loaded Setup!")
		document.getElementById("setup" + id).classList.add("loaded")
	}).catch((e) => {
		console.log(e)
		notification(e)
	})
}
document.getElementById("loadsetup").addEventListener("click", loadSetup)

function editSetupName(id, name) {
	let old_name = globalsetups[id].name
	invoke("edit_setup_name", { oldName: old_name, newName: name }).then(() => {
		notification("Edited name of current setup")
		document.getElementById("setup" + id).textContent = name
		globalsetups[id].name = name
	}).catch((e) => {
		console.log(e)
		notification(e)
	})
}
document.getElementById("setuptitle").addEventListener("keydown", (event) => {
	if (event.key === "Enter") {
		event.preventDefault()
		let id = selectedSetup
		let name = event.target.textContent
		editSetupName(id, name)
	}
})

function updateSetups() {
	selectedSetup = null
	document.getElementById("setuplist").innerHTML = ""
	globalsetups = {}
	invoke("get_setups").then((setups) => {
		notification("Updated setups")
		for (let setup of setups.sort((a, b) => (a.name < b.name) ? 1 : ((b.name < a.name) ? -1 : 0))) {
			addSetup(setup)
		}
	}).catch((e) => {
		console.log(e)
		notification(e)
	})
}
document.getElementById("refresh").addEventListener("click", updateSetups)


function selectGameDir() {
	open({ directory: true }).then(path => {
		if (path == null) {
			notification("Cancelled selection!")
			return
		}
		document.getElementById("gamepath").value = path
		localStorage.setItem("gamePath", path)
		invoke("set_config", { config: { path: path } }).then(() => {
			notification("Game Path Updated!")
		}).catch((e) => {
			console.log(e)
			notification(e)
		})
		updateSetups()
	})
}
document.getElementById("selectdir").addEventListener("click", selectGameDir)


function saveSettings() {
	notification("Settings saved")
	let gamePath = document.getElementById("gamepath").value
	let windowsCopy = document.getElementById("windowscopy").checked
	let notificationsDisabled = document.getElementById("notifdisable").checked
	localStorage.setItem("gamePath", gamePath)
	localStorage.setItem("windowsCopy", windowsCopy)
	localStorage.setItem("notificationsDisabled", notificationsDisabled)
}
document.getElementById("savesettings").addEventListener("click", saveSettings)

function loadSettings() {
	if (localStorage.length == 0) {
		selectGameDir()
		localStorage.setItem("windowsCopy", true)
		localStorage.setItem("notificationsDisabled", false)
		document.getElementById("windowscopy").checked = true
		document.getElementById("notifdisable").checked = false
		notification("First start! Thank you for using my program")
	} else {
		notification("Settings loaded")
		let gamePath = localStorage.getItem("gamePath")
		let windowsCopy = localStorage.getItem("windowsCopy")
		let notificationsDisabled = localStorage.getItem("notificationsDisabled")
		document.getElementById("gamepath").value = gamePath
		document.getElementById("windowscopy").checked = JSON.parse(windowsCopy)
		document.getElementById("notifdisable").checked = JSON.parse(notificationsDisabled)
		invoke("set_config", { config: { path: gamePath, windows_copy: windowsCopy } }).catch((e) => { notification(e) })
		updateSetups()
	}
}
document.getElementById("loadsettings").addEventListener("click", loadSettings)

function clearSettings() {
	localStorage.clear()
	close()
}
document.getElementById("clearsettings").addEventListener("click", clearSettings)

loadSettings()

document.getElementById("fireinthehole").addEventListener("click", () => {
	let audio = new Audio("fire-in-the-hole-geometry-dash.mp3")
	notification("PAINT THE WORLD WHITE")
	notification("PAINT THE WORLD RED")
	audio.preservesPitch = false
	function PAINTTHEWORLDRED(time) {
		if (audio.playbackRate > 15) {
			return
		}
		setTimeout(() => {
			PAINTTHEWORLDRED(time)
		}, time)
		audio.play()
		audio.playbackRate = 2000 / time
		time /= 1.05
	}
	document.getElementById("settingswindow").style.background = "#F7160F"
	document.getElementById("settingstitle").textContent = "PAINT THE WORLD RED"
	PAINTTHEWORLDRED(2400)
})
