interface PageContainerProps {
  children: React.ReactNode;
  className?: string;
}


const PageContainer: React.FC<PageContainerProps> = ({ children, className }) => {

    return(
        <div className={`relative min-h-full bg-[#15171C] py-6 ${className ?? ""}`}>
            {children}
        </div>
    )
}
export default PageContainer
