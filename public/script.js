window.addEventListener("load", (event) => {
    window.addEventListener('blur', (event) => {
        const instances = document.querySelector(".login-instance.selected")
        if (instances != null) {
            instances.classList.add("inactive");
        }
        const sidebar = document.querySelector(".cell.selected");
        if (sidebar != null) {
            sidebar.classList.add("inactive");
        }
        const cell = document.querySelector(".content-cell.cell-selected")
        if (cell != null) {
            cell.classList.add("inactive");
        }
        const ccell = document.querySelector(".conversation-child-selected")
        if (ccell != null) {
            ccell.classList.add("inactive");
        }
    });
    window.addEventListener('focus', (event) => {
        const instances = document.querySelector(".login-instance.selected")
        if (instances != null) {
            instances.classList.remove("inactive");
        }
        const sidebar = document.querySelector(".cell.selected");
        if (sidebar != null) {
            sidebar.classList.remove("inactive");
        }
        const cell = document.querySelector(".content-cell.cell-selected")
        if (cell != null) {
            cell.classList.remove("inactive");
        }
        const ccell = document.querySelector(".conversation-child-selected")
        if (ccell != null) {
            ccell.classList.remove("inactive");
        }
    });
});