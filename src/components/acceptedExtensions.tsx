export default function (extension: Array<string>) {
    return (
        extension.map((id) =>
            <input type="checkbox" id={id}>
                {id}
            </input>
        )
    )

}