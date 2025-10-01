module mux2to1 (
    input wire a,
    input wire b,
    input wire sel,
    output wire y
);

    always @(*) begin
        if sel == 1'b0 begin
        y <= a;
        end else begin
        y <= b;
        end
    end
endmodule

