(function () {
    const vscode = acquireVsCodeApi();

    console.log("hello from js");
}());

// Create a table element.
const table = document.createElement('table');

// Create a table head element.
const thead = table.createTHead('Registers');

// Create a table row element for the table header.
const headerRow = thead.insertRow();

// Create table header elements for the table header row.
const headerCell1 = document.createElement('th');
const headerCell2 = document.createElement('th');
const headerCell3 = document.createElement('th');
const headerCell4 = document.createElement('th');
const headerCell5 = document.createElement('th');

// Set the text content of the table header elements.
headerCell1.textContent = 'Address';
headerCell2.textContent = 'Value (+0)';
headerCell3.textContent = 'Value (+4)';
headerCell4.textContent = 'Value (+8)';
headerCell5.textContent = 'Value (+16)';

// Append the table header elements to the table header row.
headerRow.appendChild(headerCell1);
headerRow.appendChild(headerCell2);
headerRow.appendChild(headerCell3);
headerRow.appendChild(headerCell4);
headerRow.appendChild(headerCell5);

// Create a table body element.
const tbody = table.createTBody();

// Create table row elements for the table body.
const bodyRow1 = tbody.insertRow();
const bodyRow2 = tbody.insertRow();

// Create table data elements for the table body rows.
const bodyCell11 = document.createElement('td');
const bodyCell12 = document.createElement('td');
const bodyCell13 = document.createElement('td');
const bodyCell14 = document.createElement('td');
const bodyCell15 = document.createElement('td');
const bodyCell21 = document.createElement('td');
const bodyCell22 = document.createElement('td');
const bodyCell23 = document.createElement('td');
const bodyCell24 = document.createElement('td');
const bodyCell25 = document.createElement('td');

// Set the text content of the table data elements.
bodyCell11.textContent = '0x10010000';
bodyCell12.textContent = '0x00000000';
bodyCell13.textContent = '0x00000000';
bodyCell14.textContent = '0x00000000';
bodyCell15.textContent = '0x00000000';
bodyCell21.textContent = '0x00000000';
bodyCell22.textContent = '0x00000000';
bodyCell23.textContent = '0x00000000';
bodyCell24.textContent = '0x00000000';
bodyCell25.textContent = '0x00000000';

// Append the table data elements to the table body rows.
bodyRow1.appendChild(bodyCell11);
bodyRow1.appendChild(bodyCell12);
bodyRow1.appendChild(bodyCell13);
bodyRow1.appendChild(bodyCell14);
bodyRow1.appendChild(bodyCell15);
bodyRow2.appendChild(bodyCell21);
bodyRow2.appendChild(bodyCell22);
bodyRow2.appendChild(bodyCell23);
bodyRow2.appendChild(bodyCell24);
bodyRow2.appendChild(bodyCell25);

// Append the table body element to the table element.
table.appendChild(tbody);

// Append the table element to the DOM.
document.body.appendChild(table);
