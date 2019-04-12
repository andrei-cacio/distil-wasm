[![npm version](https://badge.fury.io/js/distil.svg)](https://badge.fury.io/js/distil)

# distil-wasm
This is a WebAssembly port of Elliot Jackson's Distil app (original repo: [https://github.com/elliotekj/distil](https://github.com/elliotekj/distil)

## Installation

```bash
npm i distil
```

## Development

If you want to start to play around with the code, you will need the following toolchain to make it work:

- [`rust toolchain`](https://rustwasm.github.io/book/game-of-life/setup.html)
- [`wasm-pack`](https://github.com/rustwasm/wasm-pack)

After installing everything you can use `wasm-pack build` to build and generate the npm module with the compiled wasm file.


## Usage

Because `distil` is a WebAssembly library, we will need Webpack 4 to help us easily bundle it. Luckily there is an app template for that which is called [`create-wasm-app`](https://github.com/rustwasm/create-wasm-app). To set everything up you can do the following:

```bash
mkdir distil-app
cd distil-app
npm init wasm-app
npm i && npm i distil
npm start
```

After running `npm init wasm-app` we will have a full project generated and ready to hack. In the generated `index.js` we can add the following lines of code to get a result:

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

