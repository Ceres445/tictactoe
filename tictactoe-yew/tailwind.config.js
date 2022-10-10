module.exports = {
    plugins: [require("daisyui")],
    daisyui: { themes: ["night", "cupcake"] },
    content: ["./src/**/*.rs", "./static/index.html", "./src/**/*.html", "./src/**/*.css"],
    variants: {},
    // plugins: [
    //     function ({ addComponents }) {
    //         addComponents({
    //             ".container": {
    //                 maxWidth: "100%",
    //                 "@screen sm": {
    //                     maxWidth: "640px",
    //                 },
    //                 "@screen md": {
    //                     maxWidth: "768px",
    //                 },
    //                 "@screen lg": {
    //                     maxWidth: "1000px",
    //                 },
    //                 "@screen xl": {
    //                     maxWidth: "1000px",
    //                 },
    //             },
    //         });
    //     },
    // ],
};
