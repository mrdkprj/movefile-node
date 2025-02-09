window.onload = () => {

    // window.electronAPI.onMyEventName(data => {
    //     document.getElementById("prog").value = data[0].current;
    // });

    // document.getElementById("move").addEventListener("click", () => {
    //     document.getElementById("prog").value = 0;
    //     if(navigator.userAgent.includes("Linux")) {
    //         const s = "/mnt/d/2023.mp4"
    //         const d = "/mnt/c/2023.mp4"
    //         window.electronAPI.setTitle(s, d);
    //     }else {
    //         const s = document.getElementById("s").value
    //         const d = document.getElementById("d").value
    //         if(s && d){
    //             window.electronAPI.setTitle(s, d);
    //         }
    //     }
    // })
    document.getElementById("move").addEventListener("click", () => {
        window.electronAPI.setTitle();
    });


    document.getElementById("append").addEventListener("click", () => {

        window.electronAPI.append("");

    })

    document.getElementById("reload").addEventListener("click", () => {
        window.electronAPI.reload("","");

    })

    document.getElementById("toggle").addEventListener("click", () => {
        window.electronAPI.toggle("","");

    })

    document.getElementById("open").addEventListener("click", () => {
        window.electronAPI.open();

    })

    document.getElementById("openwith").addEventListener("click", () => {
        window.electronAPI.openwith();

    })

    document.getElementById("content").addEventListener("click", () => {
        window.electronAPI.content();

    })

    document.getElementById("draggable").addEventListener("dragstart", (e) => {
        e.preventDefault();

        window.electronAPI.draggable();
    })


    document.addEventListener("dragover", e => e.preventDefault());
    // document.getElementById("multi").addEventListener("click", () => {
    //     if(navigator.userAgent.includes("Linux")) {
    //         window.electronAPI.reload(["/mnt/d/2023.mp4","/mnt/d/2024.mp4"], "/mnt/c/DevProjects");
    //     }else{
    //         window.electronAPI.reload(["D:\\a - コピー.mp4","D:\\a - コピー - コピー.mp4"], "C:\\DevProjects");
    //     }
    // })

    // window.addEventListener("keydown", e => {
    //     if(e.key == "Escape"){
    //         e.preventDefault();
    //         window.electronAPI.toggle();
    //     }
    // })
}

