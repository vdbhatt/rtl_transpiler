module simple_fsm (
    input wire clk,
    input wire reset,
    input wire start,
    output wire done,
    output wire busy,
    output wire [1:0] state_out
);

    always @(posedge clk or posedge reset) begin
        if (reset == 1'b1) begin
        state <= IDLE;
        end else begin
        state <= next_state;
        end
    end

    always @(*) begin
        case (state)
        IDLE: begin
        if (start == 1'b1) begin
        next_state <= RUNNING;
        end else begin
        next_state <= IDLE;
        end
        end
        RUNNING: begin
        next_state <= FINISH;
        end
        FINISH: begin
        next_state <= IDLE;
        end
        default: begin
        next_state <= IDLE;
        end
        endcase
    end

    assign busy = state == RUNNING ? 1'b1 : 1'b0;

    // TODO: Convert VHDL 'with...select' statement:
    // with state select
    //         state_out <= "00" when IDLE,
    //                      "01" when RUNNING,
    //                      "10" when FINISH,
    //                      "11" when others
endmodule

