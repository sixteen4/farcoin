function sendRequest(url, data, callback) {
  let xhr = new XMLHttpRequest();
  
  xhr.open("POST", url, true);
  xhr.setRequestHeader("Content-Type", "application/json");
  
  xhr.onreadystatechange = function () {
    if (xhr.readyState === 4 && xhr.status === 201) {
      let response = JSON.parse(this.responseText);
      
      callback(response);
    }
  };
  
  xhr.send(data);
}