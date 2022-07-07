import "./Message.css"

export default function Message(props) {
	return (
		<div style={{marginTop:"5px", marginBottom:"5px"}} className="msg">
			<div className="msgname"> { props.name } </div>
			<div style={{ paddingTop: "2.5px" }} className="msgcontent"> { props.content } </div>
		</div>
	)
}
