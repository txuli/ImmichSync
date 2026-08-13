interface SectionProps {
  children: React.ReactNode;
  className?: string;
}
const ImmichForm: React.FC<SectionProps> = ({ children }) => {

    return(
        <div className="bg-[#20232B] w-full max-w-2xl rounded-md border border-[#272A31] content-center mt-6 sm:mt-10">
            <div className="mx-4 sm:mx-5">{children}</div>
        </div>
    )
}
export default ImmichForm