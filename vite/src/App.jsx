import { useEffect, useRef, useState } from 'react'
import Message from "./Message.jsx"
import './App.css'
import { Button, Group, ScrollArea, TextInput } from '@mantine/core';
import { useForm } from '@mantine/hooks';

function getParameterByName(name, url = window.location.href) {
    name = name.replace(/[\[\]]/g, '\\$&');
    var regex = new RegExp('[?&]' + name + '(=([^&#]*)|&|#|$)'),
        results = regex.exec(url);
    if (!results) return null;
    if (!results[2]) return '';
    return decodeURIComponent(results[2].replace(/\+/g, ' '));
}

function App() {
	let [content, setContent] = useState([]);
	let [url, _] = useState("localhost:3030");

	const viewport = useRef();

	const scrollToBottom = () =>
		viewport.current.scrollTo({ top: viewport.current.scrollHeight, behavior: 'smooth' });

	const form = useForm({
		initialValues: {
			message: ''
		}
	})

	useEffect(() => {
		let w = new EventSource(`${location.protocol.slice(0, -1)}://${url}/ticks_${getParameterByName("p")}`);
		w.onmessage = function(event) {
			let v = event.data.split("\n").reverse()
			setContent(v)
			scrollToBottom()
		}
	}, [])

	return (
		<div
			style={{
				backgroundColor: "#25283b",
				height: "100%",
				width: "100%",
				display: "grid",
				justifyContent: "center",
				alignContent: "center"
			}}
		>
			<ScrollArea offsetScrollbars viewportRef={viewport} style={{ height: "80vh" }}>
				{content.map((s, i) => {
					return (
						<Message
							key={i.toString()}
							name={s.match(/(\[\w+\])/g)}
							content={/(\[\w+\]): (.+)/g.exec(s)[2]}
						/>
					)
				})}

			</ScrollArea>
			<form onSubmit={form.onSubmit(({ message }) => {
				console.log(message)
				fetch(`http://${url}/sendfoo/${message}/${getParameterByName("p")}%20${getParameterByName("name") || "Unknown"}`).then(_ => {
					scrollToBottom()
				})
			})}>
				<TextInput
					required
					placeholder="Hello, friends"
					style={{ paddingTop: "10px" }}
					{...form.getInputProps('message')}
				/>
				<Group position='right'>
					<Button type="submit">Submit</Button>
				</Group>
			</form>
		</div>
	)
}

export default App
