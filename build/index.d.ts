declare namespace MoveFile {
    function mv(sourceFile:string, destFile:string, callback:any): number;
    function mvSync(sourceFile:string, destFile:string): number;
    function cancel(id:number):boolean;
}

export = MoveFile;