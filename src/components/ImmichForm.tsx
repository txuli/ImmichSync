interface SectionProps {
  children: React.ReactNode;
  className?: string;
}
const ImmichForm: React.FC<SectionProps> = ({ children }) => {

    return(
        <div className=" bg-[#20232B] w-3/5 rounded-md border border-[#272A31] content-center mt-10 ">
            <div className="mx-5">{children}</div>
        </div>
    )
}
export default ImmichForm