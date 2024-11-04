window.onload = () => {

    window.electronAPI.onMyEventName(data => {
        document.getElementById("prog").value = data[0].current;
    });

    document.getElementById("move").addEventListener("click", () => {
        document.getElementById("prog").value = 0;
        if(navigator.userAgent.includes("Linux")) {
            const s = "/mnt/d/2023.mp4"
            const d = "/mnt/c/2023.mp4"
            window.electronAPI.setTitle(s, d);
        }else {
            const s = document.getElementById("s").value
            const d = document.getElementById("d").value
            if(s && d){
                window.electronAPI.setTitle(s, d);
            }
        }
    })


    document.getElementById("cancel").addEventListener("click", () => {
            window.electronAPI.toggle();

    })

    document.getElementById("trash").addEventListener("click", () => {
        const s = document.getElementById("s").value
        window.electronAPI.append(s);

    })

    document.getElementById("multi").addEventListener("click", () => {
        if(navigator.userAgent.includes("Linux")) {
            window.electronAPI.reload(["/mnt/d/2023.mp4","/mnt/d/2024.mp4"], "/mnt/c/DevProjects");
        }else{
            window.electronAPI.reload(["D:\\2023.mp4","D:\\2024.mp4"], "C:\\DevProjects");
        }
    })

    window.addEventListener("keydown", e => {
        if(e.key == "Escape"){
            e.preventDefault();
            window.electronAPI.toggle();
        }
    })
}