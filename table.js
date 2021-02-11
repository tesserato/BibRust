// declare var tabledata;
var Tabulator = require('tabulator-tables');
var selectedtags = new Set([]);
var fields = {};
for (var _i = 0, tabledata_1 = tabledata; _i < tabledata_1.length; _i++) {
    var e = tabledata_1[_i];
    // console.log(e);
    for (var f in e) {
        // console.log(f);
        if (fields[f]) {
            fields[f] += 1;
        }
        else {
            fields[f] = 1;
        }
    }
}
var sortedfields = Object.keys(fields).sort(function (a, b) { return fields[b] - fields[a]; });
// console.log(sortedfields);
var hcm = [
    {
        label: "Hide this column",
        action: function (e, column) {
            column.hide();
        }
    },
    {
        label: "Show all columns",
        action: function (e, column) {
            for (var _i = 0, _a = column.getTable().getColumns(); _i < _a.length; _i++) {
                var col = _a[_i];
                col.show();
            }
        }
    },
    {
        label: "Delete this column",
        action: function (e, column) {
            var conf = confirm("Really? This can't be undone!");
            if (conf) {
                column.delete();
            }
        }
    },
    {
        label: "Add new column",
        action: function (e, column) {
            var _a = window.prompt("field Title").split(' '), field = _a[0], title = _a[1];
            table.addColumn({ title: title, field: field }, true, column);
            table.updateColumnDefinition(field, { downloadTitle: field, width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm });
        }
    },
];
function customHeaderFilter(headerValue, rowValue, rowData, filterParams) {
    //headerValue - the value of the header filter element
    //rowValue - the value of the column in this row
    //rowData - the data for the row being filtered
    //filterParams - params object passed to the headerFilterFuncParams property
    if (headerValue && !rowValue) {
        return false;
    }
    else {
        return true;
    }
}
ww = 200;
var types = { values: ["article", "inproceedings", "unpublished", "incollection", "book", "report", "proceedings", "collection", "misc", "online", "thesis"] };
var coldef = [
    {
        title: "OK?",
        field: "reviewed",
        titleDownload: "reviewed",
        headerFilter: true,
        hozAlign: "center",
        formatter: "tickCross",
        sorter: "boolean",
        formatterParams: {
            allowEmpty: false,
            allowTruthy: true,
            tickElement: "&starf;",
            crossElement: false,
        },
        cellClick: function (e, cell) {
            cell.setValue(!cell.getValue(), true);
        },
        headerFilterFunc: customHeaderFilter,
    },
    { title: "Type", field: "type", titleDownload: "type", editor: "select", editorParams: types, headerFilter: "input", headerContextMenu: hcm, validator: "required" },
    { title: "Key", field: "key", titleDownload: "key", editor: "input", headerFilter: "input", headerContextMenu: hcm, validator: "required", validator: "unique" },
    { title: "Author", field: "author", titleDownload: "author", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
    { title: "Editor", field: "editor", titleDownload: "editor", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
    { title: "Title", field: "title", titleDownload: "title", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
    { title: "Year", field: "year", titleDownload: "year", editor: "input", headerFilter: "input", headerContextMenu: hcm },
    { title: "Journal", field: "journal", titleDownload: "journal", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
    { title: "url", field: "url", titleDownload: "url", width: ww, editor: "input", headerFilter: "input",
        formatter: "link",
        formatterParams: { target: "_blank" },
        headerContextMenu: hcm
    },
    { title: "File", field: "file", titleDownload: "file", width: ww, editor: "input", headerFilter: "input", formatter: "link", formatterParams: { target: "_blank" }, headerContextMenu: hcm },
    { title: "Abstract", field: "abstract", titleDownload: "abstract", width: ww, editor: "input", headerFilter: "input", headerContextMenu: hcm },
    { title: "Tags", field: "tags", titleDownload: "tags", width: ww / 2, editor: "input", headerFilter: "input", headerContextMenu: hcm },
];
var existingfields = new Set([]);
for (var _a = 0, coldef_1 = coldef; _a < coldef_1.length; _a++) {
    o = coldef_1[_a];
    existingfields.add(o.field);
}
for (var _b = 0, sortedfields_1 = sortedfields; _b < sortedfields_1.length; _b++) {
    field = sortedfields_1[_b];
    if (!existingfields.has(field)) {
        e = {
            title: field[0].toUpperCase() + field.slice(1),
            field: field,
            titleDownload: field,
            width: ww / 2,
            editor: "input",
            headerFilter: "input",
            headerContextMenu: hcm
        };
        coldef.push(e);
    }
}
function fullscreen() {
    if (elem.requestFullscreen) {
        elem.requestFullscreen();
    }
    else if (elem.webkitRequestFullscreen) { /* Safari */
        elem.webkitRequestFullscreen();
    }
    else if (elem.msRequestFullscreen) { /* IE11 */
        elem.msRequestFullscreen();
    }
}
function updatealltags() {
    alltags = new Set([]);
    for (var _i = 0, tabledata_2 = tabledata; _i < tabledata_2.length; _i++) {
        var e_1 = tabledata_2[_i];
        if (e_1['tags']) {
            for (var _a = 0, _b = e_1['tags'].split(","); _a < _b.length; _a++) {
                var tag = _b[_a];
                alltags.add(tag);
            }
        }
    }
    alltags = Array.from(alltags).sort();
    // console.log(alltags)
    parentdiv = document.getElementById("tags");
    parentdiv.innerHTML = "";
    for (var _c = 0, alltags_1 = alltags; _c < alltags_1.length; _c++) {
        var tag = alltags_1[_c];
        div = document.createElement("div");
        txt = document.createTextNode(tag);
        div.appendChild(txt);
        div.classList.add("tag");
        if (selectedtags.has(tag)) {
            div.classList.add("selectedtag");
        }
        else {
            div.classList.add("deselectedtag");
        }
        div.onclick = handleLeftClick;
        div.oncontextmenu = handleRightCLick;
        parentdiv.appendChild(div);
    }
    var btn = document.createElement("BUTTON"); // Create a <button> element
    btn.innerHTML = "Clear tags"; // Insert text
    btn.classList.add("btn");
    btn.onclick = function (event) {
        selectedtags.clear();
        table.clearFilter();
        for (var _i = 0, _a = document.getElementsByClassName("tag"); _i < _a.length; _i++) {
            e = _a[_i];
            e.classList.remove("selectedtag");
            e.classList.add("deselectedtag");
        }
    };
    parentdiv.appendChild(btn);
    var btn = document.createElement("BUTTON"); // Create a <button> element
    btn.innerHTML = "Fullscreen"; // Insert text
    btn.classList.add("btn");
    btn.onclick = function (event) {
        var elem = document.documentElement;
        if (elem.requestFullscreen) {
            elem.requestFullscreen();
        }
        else if (elem.webkitRequestFullscreen) { /* Safari */
            elem.webkitRequestFullscreen();
        }
        else if (elem.msRequestFullscreen) { /* IE11 */
            elem.msRequestFullscreen();
        }
    };
    parentdiv.appendChild(btn);
    var btn = document.createElement("BUTTON"); // Create a <button> element
    btn.innerHTML = "Download as .csv"; // Insert text
    btn.classList.add("btn");
    btn.onclick = function (event) {
        // var elem = document.documentElement;
        table.download("csv", "bibliography.csv", { bom: true });
    };
    parentdiv.appendChild(btn);
    var btn = document.createElement("BUTTON"); // Create a <button> element
    btn.innerHTML = "Show all columns"; // Insert text
    btn.classList.add("btn");
    btn.onclick = function (event) {
        for (var _i = 0, _a = table.getColumns(); _i < _a.length; _i++) {
            col = _a[_i];
            col.show();
        }
    };
    parentdiv.appendChild(btn);
    // var btn = document.createElement("BUTTON");   // Create a <button> element
    // btn.innerHTML = "Save JSON";                   // Insert text
    // btn.classList.add("btn");
    // btn.onclick = function (event) {
    //   table.download("json", "bibliography.json");
    // };
    // parentdiv.appendChild(btn);
}
var table = new Tabulator("#table", {
    // keybindings:{
    //   "undo" : "ctrl + z",
    //   "redo" : "ctrl + y",
    // },
    cellEdited: function (cell) {
        if (cell.getColumn().getField() == "tags") {
            updatealltags();
        }
    },
    rowContextMenu: [
        {
            label: "Toggle freeze row",
            action: function (e, row) {
                if (row.isFrozen()) {
                    row.unfreeze();
                }
                else {
                    row.freeze();
                }
            }
        },
        {
            label: "Insert row above",
            action: function (e, row) {
                table.addData([{}], true, row.getElement());
            }
        },
        {
            label: "Insert row below",
            action: function (e, row) {
                table.addData([{}], false, row.getElement());
            }
        },
        {
            label: "Delete Row",
            action: function (e, row) {
                row.delete();
            }
        },
        {
            label: "Download data as .csv",
            action: function (e, row) {
                table.download("csv", "bibliography.csv", { bom: true });
            }
        },
    ],
    // selectable:true,
    persistenceID: "bibtable",
    persistence: {
        sort: true,
        filter: true,
        columns: true //Enable column layout persistence
    },
    data: tabledata,
    downloadRowRange: "all",
    layout: "fitDataFill",
    // responsiveLayout:"hide",  //hide columns that dont fit on the table
    tooltips: true,
    addRowPos: "top",
    history: true,
    pagination: "local",
    paginationSize: 100,
    movableColumns: true,
    resizableRows: true,
    // autoColumns:true,
    // initialSort: [             //set the initial sort order of the data
    //   { column: "name", dir: "asc" },
    // ],
    columns: coldef,
});
updatealltags(table.getColumn("tags"));
function customFilter(data, selectedtags) {
    if (!data.tags) {
        return false;
    }
    else {
        var tags = new Set(data.tags.split(","));
        for (var _i = 0, selectedtags_1 = selectedtags; _i < selectedtags_1.length; _i++) {
            var elem = selectedtags_1[_i];
            if (!tags.has(elem)) {
                return false;
            }
        }
        return true;
    }
}
function handleLeftClick(event) {
    var tag = event.target.textContent;
    if (selectedtags.has(tag)) {
        selectedtags.delete(tag);
        event.target.classList.remove("selectedtag");
        event.target.classList.add("deselectedtag");
        if (selectedtags.length == 0) {
            table.clearFilter();
        }
        else {
            table.setFilter(customFilter, selectedtags);
        }
        // table.update();
    }
    else {
        selectedtags.add(tag);
        event.target.classList.remove("deselectedtag");
        event.target.classList.add("selectedtag");
        table.setFilter(customFilter, selectedtags);
        // table.update();
    }
}
function handleRightCLick(event) {
    event.preventDefault();
    // console.log(event)
    var oldtag = event.target.textContent;
    var newtag = prompt("Rename tag ( Warning: Can't be undone! )", oldtag);
    // console.log(newtag)
    if (newtag != null) {
        console.log("renaming", oldtag, " to ", newtag);
        // console.log(table.getColumn("tags").getCells());
        for (var _i = 0, _a = table.getColumn("tags").getCells(); _i < _a.length; _i++) {
            c = _a[_i];
            // console.log(c.getValue())
            if (c.getValue()) {
                c.setValue(c.getValue().replace(oldtag, newtag).split(',').map(function (e) { return e.trim(); }).filter(function (e) { return e; }).join(','));
            }
        }
        if (selectedtags.has(oldtag)) {
            selectedtags.delete(oldtag);
            selectedtags.add(newtag);
        }
        updatealltags();
    }
}
document.addEventListener('keydown', hadleKey);
function hadleKey(e) {
    // console.log(e);
    if (e.ctrlKey) {
        switch (e.key) {
            case "z":
                // console.log("undo");
                table.undo();
                break;
            case "y":
                // console.log("redo");
                table.redo();
                break;
            case "s":
                // console.log("redo");
                e.preventDefault();
                table.download("csv", "bibliography.csv", { bom: true });
                break;
            default:
            // code block
        }
    }
}
document.body.appendChild(document.getElementsByClassName("tabulator-footer")[0]);
document.getElementById("splash").remove();
//# sourceMappingURL=table.js.map