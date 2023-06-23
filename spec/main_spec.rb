describe "database" do
    before do
        `cargo b`
    end
    
    def run_script(commands)
        raw_output = nil
        IO.popen("./target/release/rs-sqlite", "r+") do |pipe|
            commands.each do |command|
                begin
                    pipe.puts command
                rescue Errno::EPIPE
                    break
                end
        end
    
            pipe.close_write
        
            # Read entire output
            raw_output = pipe.gets(nil)
        end
        raw_output.split("\n")
    end
    
    it 'inserts and retrieves a row' do
        result = run_script([
            "insert 1 user1 person1@example.com",
            "select",
            ".exit",
        ])
        expect(result).to match_array([
            "Executed.",
            'db > (1, "user1", "person1@example.com")',
            "db > Bye~", 
            "db > Executed.",
        ])
    end
end
