# distil-wasm
This is a WebAssembly port of Elliot Jackson's Distil app (original repo: [https://github.com/elliotekj/distil](https://github.com/elliotekj/distil)

## Usage

```bash
npm init wasm-app
cd wasm-app
npm i distil
npm start
```

In the generated `index.js` we can add the following lines of code to get a result:

```javascript
import { distil } from 'distil';

const renderColors = async (imageName) => {
	const response = await fetch(imageName);
	const blob = await response.blob();
	const result = await new Promise((resolve, reject) => {
	  const reader = new FileReader();
	  reader.onloadend = () => {
	    if (reader.result instanceof ArrayBuffer) {
	      return resolve(reader.result);
	    } else {
	      return reject(new Error("Could not create arraybuffer"));
	    }
	  };
	  reader.onerror = reject;
	  reader.readAsArrayBuffer(blob);
	});

		
	const colors = distil(new Uint8Array(result));

	const container = document.body;
	container.innerHTML = '';
	console.log(colors);
	colors.forEach(([r, g, b]) => {
		const span = document.createElement('span');
		span.style.backgroundColor = `rgb(${r}, ${g}, ${b})`;
		span.style.width="100px";
		span.style.height="100px";
		span.style.display="inline-block";
		container.appendChild(span);
	});
};

window.distil = renderColors;

```

and for an example like this;

![](./images/img-1.jpg?raw=true)

you can run the code below into the browser's console:

```javascript
distil('img-1.jpg');
```

you should get the following output:

![](./images/colors.png?raw=true)

